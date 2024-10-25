use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use bytes::{Bytes, BytesMut};
use eyre::{Context, Result};
use ratatui::{layout::Rect, style::Color, Frame};
use tokio::sync::watch;

pub fn update_channel() -> (PngSender, PngReceiver) {
    watch::channel((Bytes::new(), SystemTime::now()))
}

pub type PngReceiver = watch::Receiver<(Bytes, SystemTime)>;
pub type PngSender = watch::Sender<(Bytes, SystemTime)>;

pub struct PngBuilder {
    height: u32,
    width: u32,
    data: Vec<u8>,
}

impl PngBuilder {
    pub fn new(frame: &mut Frame, area: &Rect) -> Self {
        let buffer = frame.buffer_mut();

        let data = area.positions().fold(vec![], |mut png, position| {
            let color = buffer
                .cell(position)
                .map(|cell| cell.fg)
                .unwrap_or(Color::Black);

            let (r, g, b) = match color {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (0, 0, 0),
            };

            png.push(r);
            png.push(g);
            png.push(b);

            png
        });

        Self {
            data,
            height: area.height as u32,
            width: area.width as u32,
        }
    }

    pub fn build(self) -> Result<Bytes> {
        let png_writer = PngWriter::default();

        let mut encoder = png::Encoder::new(png_writer.clone(), self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder
            .write_header()
            .wrap_err("Unable to write PNG header")?;

        writer
            .write_image_data(&self.data)
            .wrap_err("Unable to write PNG data")?;
        writer.finish().wrap_err("Unable to write PNG")?;

        Ok(png_writer.into())
    }
}

#[derive(Clone, Default)]
struct PngWriter {
    data: Arc<Mutex<BytesMut>>,
}

impl From<PngWriter> for Bytes {
    fn from(png: PngWriter) -> Self {
        let png = png.data.lock().unwrap();

        png.clone().freeze()
    }
}

impl std::io::Write for PngWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        {
            let mut data = self.data.lock().unwrap();
            data.extend_from_slice(buf);
        }

        std::io::Result::Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::Result::Ok(())
    }
}
