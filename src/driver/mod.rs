#[cfg(target_os = "windows")]
pub mod dxgi;

use crate::{
  errors::{DisplayError, FrameError},
  Frame,
};
use std::ops::Deref;

pub struct BoxDisplayDriver(Box<dyn DisplayDriver>);

impl BoxDisplayDriver {
  pub fn new<D: 'static + DisplayDriver>(driver: D) -> Self {
    Self(Box::new(driver))
  }
}

impl Deref for BoxDisplayDriver {
  type Target = dyn DisplayDriver;

  fn deref(&self) -> &Self::Target {
    self.0.as_ref()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayHandle(pub(crate) usize);

impl Deref for DisplayHandle {
  type Target = usize;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub trait DisplayDriver {
  fn name(&self) -> &'static str;

  fn get_all(&self) -> Result<Vec<DisplayHandle>, DisplayError>;
  fn get_primary(&self) -> Result<Option<DisplayHandle>, DisplayError>;

  fn get_display_frame(&self, display: DisplayHandle) -> Result<Frame<'_>, FrameError>;
  fn get_display_width(&self, display: DisplayHandle) -> Result<usize, DisplayError>;
  fn get_display_height(&self, display: DisplayHandle) -> Result<usize, DisplayError>;
}
