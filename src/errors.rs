#[derive(thiserror::Error, Debug, Clone)]
pub enum FrameError {
  #[error("The operation needs to block to complete, but the blocking operation was requested to not occur.")]
  WouldBlock,
  #[cfg(target_os = "windows")]
  #[error(transparent)]
  Dxgi(crate::driver::dxgi::errors::FrameError),
}

#[cfg(target_os = "windows")]
impl From<crate::driver::dxgi::errors::FrameError> for FrameError {
  fn from(inner: crate::driver::dxgi::errors::FrameError) -> Self {
    match inner {
      crate::driver::dxgi::errors::FrameError::WouldBlock => Self::WouldBlock,
      _ => Self::Dxgi(inner),
    }
  }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, PartialOrd)]
pub enum DisplayError {}

#[derive(thiserror::Error, Debug, Clone, PartialEq, PartialOrd)]
pub enum DriverError {}
