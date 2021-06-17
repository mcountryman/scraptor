use crate::driver::dx11::frame::D3D11TextureFrame;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Frame<'frame> {
  Memory {
    fmt: PixelFormat,
    buf: &'frame [u8],
  },
  #[cfg(target_os = "windows")]
  D3D11Texture(D3D11TextureFrame<'frame>),
}

impl<'frame> Frame<'frame> {
  pub fn as_bytes(&self) -> anyhow::Result<Cow<'frame, [u8]>> {
    match self {
      Self::Memory { buf, .. } => Ok(Cow::from(*buf)),
      #[cfg(target_os = "windows")]
      Self::D3D11Texture(frame) => Ok(Cow::from(frame.as_bytes()?)),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
  Bgra,
}

#[cfg(target_os = "windows")]
impl<'frame> From<D3D11TextureFrame<'frame>> for Frame<'frame> {
  fn from(frame: D3D11TextureFrame<'frame>) -> Self {
    Self::D3D11Texture(frame)
  }
}
