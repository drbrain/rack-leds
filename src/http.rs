use std::{future::Future, net::SocketAddr, pin::Pin};

use bytes::Bytes;
use eyre::Result;
use http_body_util::Full;
use hyper::{
    body::Incoming, header::ALLOW, server::conn::http1, service::Service, Method, Request,
    Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::watch,
};
use tracing::instrument;

pub struct Http {
    addr: SocketAddr,
    png: watch::Receiver<Bytes>,
}

impl Http {
    pub fn new(addr: SocketAddr, png: watch::Receiver<Bytes>) -> Result<Self> {
        Ok(Self { addr, png })
    }

    pub async fn run(self) -> Result<()> {
        let Self { addr, png } = self;

        let listener = TcpListener::bind(addr).await?;
        let mut task_id = 0usize;

        let service = PngService::new(png);

        loop {
            let (stream, _) = listener.accept().await?;

            let task_name = format!("http {task_id}");

            let io = TokioIo::new(stream);
            let service = service.clone();

            tokio::task::Builder::new()
                .name(&task_name)
                .spawn(handle_client(task_id, io, service))?;

            task_id += 1;
        }
    }
}

#[instrument(level = "DEBUG", skip_all, ret, fields(id = _id))]
async fn handle_client(_id: usize, io: TokioIo<TcpStream>, service: PngService) -> Result<()> {
    http1::Builder::new().serve_connection(io, service).await?;

    Ok(())
}

#[derive(Clone)]
struct PngService {
    png: watch::Receiver<Bytes>,
}

impl PngService {
    fn new(png: watch::Receiver<Bytes>) -> Self {
        Self { png }
    }
}

impl Service<Request<Incoming>> for PngService {
    type Response = Response<Full<Bytes>>;

    type Error = hyper::Error;

    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let (&Method::GET | &Method::HEAD) = req.method() else {
            return Box::pin(async { Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(ALLOW, "GET, HEAD")
                .body(Full::new(Bytes::new())).unwrap())});
        };

        if req.uri().path() != "/current.png" {
            return Box::pin(async {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::new()))
                    .unwrap())
            });
        }

        let current_png = self.png.borrow().clone();

        Box::pin(async {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(current_png))
                .unwrap())
        })
    }
}
