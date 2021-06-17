#[derive(thiserror::Error, Debug, Clone)]
pub enum FrameError {
  #[error("The operation needs to block to complete, but the blocking operation was requested to not occur.")]
  WouldBlock,
  #[error("Failed to acquire frame `{0}`")]
  AcquireFrame(windows::Error),
  #[error("Failed to release frame `{0}`")]
  ReleaseFrame(windows::Error),
  #[error("Unexpected error `{0}`")]
  Unexpected(#[from] windows::Error),
}
