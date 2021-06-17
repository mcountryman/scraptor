#[cfg(target_os = "windows")]
pub mod bindings;
pub mod driver;
pub mod errors;
pub mod frame;

// use driver::{BoxDisplayDriver, DisplayDriver, DisplayHandle};
use errors::{DisplayError, FrameError};
use frame::Frame;

pub type DisplayBox = Box<dyn Display>;

pub trait Display {
  fn width(&self) -> Result<usize, DisplayError>;
  fn height(&self) -> Result<usize, DisplayError>;
  fn frame(&mut self) -> Result<Frame<'_>, FrameError>;
}

pub trait DisplayDriver {
  type Display: 'static + Display + Sized;

  fn name(&self) -> &'static str;

  fn all(&self) -> Result<Vec<Self::Display>, DisplayError>;
  fn primary(&self) -> Result<Option<Self::Display>, DisplayError>;
}

pub type DisplayDriverBox = Box<dyn DisplayDriverDyn>;

pub trait DisplayDriverDyn {
  fn name(&self) -> &'static str;

  fn all(&self) -> Result<Vec<DisplayBox>, DisplayError>;
  fn primary(&self) -> Result<Option<DisplayBox>, DisplayError>;
}

impl<D: DisplayDriver> DisplayDriverDyn for D {
  fn name(&self) -> &'static str {
    self.name()
  }

  fn all(&self) -> Result<Vec<DisplayBox>, DisplayError> {
    self.all().map(|displays| {
      displays
        .into_iter()
        .map(|display| -> Box<dyn Display> { Box::new(display) })
        .collect()
    })
  }

  fn primary(&self) -> Result<Option<DisplayBox>, DisplayError> {
    Ok(
      self
        .primary()?
        .map(|display| -> Box<dyn Display> { Box::new(display) }),
    )
  }
}

pub fn drivers() -> Result<(), Vec<DisplayDriverBox>> {
  todo!()
}

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// pub struct Displays<D> {
//   driver: Arc<D>,
// }

// impl<D: DisplayDriver> Displays<D> {
//   pub fn new(driver: D) -> Self {
//     Self {
//       driver: Arc::new(driver),
//     }
//   }

//   pub fn all(&self) -> Result<Vec<Display<D>>, DisplayError> {
//     Ok(
//       self
//         .driver
//         .get_all()?
//         .into_iter()
//         .map(|handle| Display {
//           handle,
//           driver: self.driver.clone(),
//         })
//         .collect(),
//     )
//   }

//   pub fn primary(&self) -> Result<Option<Display<D>>, DisplayError> {
//     Ok(self.driver.get_primary()?.map(|handle| Display {
//       handle,
//       driver: self.driver.clone(),
//     }))
//   }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct Frame<'buf> {
//   fmt: PixelFormat,
//   buf: &'buf [u8],
// }

// impl<'buf> Frame<'buf> {
//   pub fn format(&self) -> PixelFormat {
//     self.fmt
//   }

//   pub fn as_bytes(&self) -> &'buf [u8] {
//     self.buf
//   }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub enum PixelFormat {
//   Bgra,
// }

// #[derive(Debug, Clone, PartialEq, PartialOrd)]
// pub struct Display<D: ?Sized> {
//   handle: DisplayHandle,
//   driver: Arc<D>,
// }

// impl<D: DisplayDriver> Display<D> {
//   pub fn frame(&self) -> Result<Frame<'_>, FrameError> {
//     self.driver.get_display_frame(self.handle)
//   }

//   pub fn width(&self) -> Result<usize, DisplayError> {
//     self.driver.get_display_width(self.handle)
//   }

//   pub fn height(&self) -> Result<usize, DisplayError> {
//     self.driver.get_display_height(self.handle)
//   }
// }

// pub fn displays() -> Result<BoxDisplayDriver, DriverError> {
//   todo!()
// }

// pub fn drivers() -> Result<Vec<BoxDisplayDriver>, DriverError> {
//   todo!()
// }
