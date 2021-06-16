pub mod driver;
pub mod errors;

use driver::{BoxDisplayDriver, DisplayDriver, DisplayHandle};
use errors::{DisplayError, DriverError, FrameError};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Displays<D> {
  driver: Arc<D>,
}

impl<D: DisplayDriver> Displays<D> {
  pub fn new(driver: D) -> Self {
    Self {
      driver: Arc::new(driver),
    }
  }

  pub fn all(&self) -> Result<Vec<Display<D>>, DisplayError> {
    Ok(
      self
        .driver
        .get_all()?
        .into_iter()
        .map(|handle| Display {
          handle,
          driver: self.driver.clone(),
        })
        .collect(),
    )
  }

  pub fn primary(&self) -> Result<Option<Display<D>>, DisplayError> {
    Ok(self.driver.get_primary()?.map(|handle| Display {
      handle,
      driver: self.driver.clone(),
    }))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame<'buf> {
  fmt: PixelFormat,
  buf: &'buf [u8],
}

impl<'buf> Frame<'buf> {
  pub fn format(&self) -> PixelFormat {
    self.fmt
  }

  pub fn as_bytes(&self) -> &'buf [u8] {
    self.buf
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
  Bgra,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Display<D: ?Sized> {
  handle: DisplayHandle,
  driver: Arc<D>,
}

impl<D: DisplayDriver> Display<D> {
  pub fn frame(&self) -> Result<Frame<'_>, FrameError> {
    self.driver.get_display_frame(self.handle)
  }

  pub fn width(&self) -> Result<usize, DisplayError> {
    self.driver.get_display_width(self.handle)
  }

  pub fn height(&self) -> Result<usize, DisplayError> {
    self.driver.get_display_height(self.handle)
  }
}

pub fn displays() -> Result<BoxDisplayDriver, DriverError> {
  todo!()
}

pub fn drivers() -> Result<Vec<BoxDisplayDriver>, DriverError> {
  todo!()
}
