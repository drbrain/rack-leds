use std::{future::Future, net::SocketAddr, pin::Pin, time::Duration};

use bytes::Bytes;
use eyre::Result;
use http_body_util::Full;
use httpdate::fmt_http_date;
use hyper::{
    body::Incoming, header::ALLOW, server::conn::http1, service::Service, Method, Request,
    Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, instrument};

use crate::png_builder::PngReceiver;

pub struct Http {
    addr: SocketAddr,
    png: PngReceiver,
    period: Duration,
}

impl Http {
    pub fn new(addr: SocketAddr, png: PngReceiver, period: Duration) -> Result<Self> {
        Ok(Self { addr, png, period })
    }

    #[instrument(name = "http", skip_all, fields(addr = ?self.addr))]
    pub async fn run(self) -> Result<()> {
        let Self { addr, png, period } = self;

        let listener = TcpListener::bind(addr).await?;

        info!("listening");
        let mut task_id = 0usize;

        let service = PngService::new(png, period);

        loop {
            let (stream, _) = listener.accept().await?;

            let task_name = format!("http {task_id}");

            let io = TokioIo::new(stream);
            let service = service.clone();

            tokio::task::Builder::new()
                .name(&task_name)
                .spawn(http_client(task_id, io, service))?;

            task_id += 1;
        }
    }
}

#[instrument(level = "DEBUG", skip_all, ret, fields(id = _id))]
async fn http_client(_id: usize, io: TokioIo<TcpStream>, service: PngService) -> Result<()> {
    http1::Builder::new()
        .auto_date_header(true)
        .serve_connection(io, service)
        .await?;

    Ok(())
}

#[derive(Clone)]
struct PngService {
    png: PngReceiver,
    period: Duration,
}

impl PngService {
    fn new(png: PngReceiver, period: Duration) -> Self {
        Self { png, period }
    }
}

impl Service<Request<Incoming>> for PngService {
    type Response = Response<Full<Bytes>>;

    type Error = hyper::Error;

    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        debug!(method = ?req.method(), uri = ?req.uri());

        let (&Method::GET | &Method::HEAD) = req.method() else {
            return Box::pin(async {
                Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .header(ALLOW, "GET, HEAD")
                    .body(Full::new(Bytes::new()))
                    .unwrap())
            });
        };

        if req.uri().path() == "/" {
            let refresh = self.period.as_secs();

            let body = Bytes::from(format!(
                "<!DOCTYPE html>
<head>
<title>Rack LEDS</title>
<meta http-equiv=\"refresh\" content=\"{refresh}\">
<body style=\"background: black\">
<img style=\"width: 100%\" src=\"/current.png\">
"
            ));

            return Box::pin(async move {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(body))
                    .unwrap())
            });
        }

        if req.uri().path() != "/current.png" {
            return Box::pin(async {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::new()))
                    .unwrap())
            });
        }

        let (current_png, updated) = self.png.borrow().clone();

        let expires = updated + self.period;

        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Expires", fmt_http_date(expires))
                .body(Full::new(current_png))
                .unwrap())
        })
    }
}
