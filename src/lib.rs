pub mod driver;
pub mod errors;

use errors::{DisplayError, FrameError};
use std::borrow::Cow;

/// Provides access to displays
pub trait DisplayDriver<'buf> {
  type Display: 'static + Display<'buf> + Sized;

  /// The name of the display driver
  fn name(&self) -> &'static str;
  /// Gets all displays
  fn all(&self) -> Result<Vec<Self::Display>, DisplayError>;
  /// Gets the primary display
  fn primary(&self) -> Result<Option<Self::Display>, DisplayError>;
}
/// A display that can be screen captured
pub trait Display<'buf> {
  type Frame: Frame<'buf>;

  /// The width of the display
  fn width(&self) -> Result<usize, DisplayError>;
  /// The height of the display
  fn height(&self) -> Result<usize, DisplayError>;
  /// Gets a screen capture frame
  fn frame(&'buf mut self) -> Result<Self::Frame, FrameError>;
}

/// A screen capture frame.
pub trait Frame<'buf> {
  /// Gets rectangles that changed since last frame
  fn dirty(&self) -> Vec<DirtyRect>;

  /// Gets rectangles that moved since last frame
  fn moved(&self) -> Vec<MovedRect>;

  /// The pixel format of the frame
  fn format(&self) -> FrameFormat;

  /// The pixel data of the frame
  fn as_bytes(&self) -> anyhow::Result<Cow<'buf, [u8]>>;
}

/// Pixel data format
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrameFormat {
  B8G8R8A8,
}

/// An area where pixels have changed since the last frame capture
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirtyRect {
  pub top: i32,
  pub left: i32,
  pub right: i32,
  pub bottom: i32,
}

impl DirtyRect {
  pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
    Self {
      top,
      left,
      right,
      bottom,
    }
  }
}

/// A point where an area of pixel moved to since the last frame capture
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovedPoint {
  pub x: i32,
  pub y: i32,
}

impl MovedPoint {
  pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}

/// An area where pixels have moved to a specified point since the last frame capture
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovedRect {
  pub to: DirtyRect,
  pub from: MovedPoint,
}

impl MovedRect {
  pub fn new(to: DirtyRect, from: MovedPoint) -> Self {
    Self { to, from }
  }
}
