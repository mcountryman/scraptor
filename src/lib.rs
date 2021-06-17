#[cfg(target_os = "windows")]
pub mod bindings;
pub mod driver;
pub mod errors;

use errors::{DisplayError, FrameError};
use std::borrow::Cow;

pub trait Frame<'buf> {
  fn dirty(&self) -> Vec<DirtyRect>;
  fn moved(&self) -> Vec<MovedRect>;
  fn format(&self) -> FrameFormat;

  fn as_bytes(&self) -> anyhow::Result<Cow<'buf, [u8]>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirtyRect {
  top: i32,
  left: i32,
  right: i32,
  bottom: i32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovedPoint {
  x: i32,
  y: i32,
}

impl MovedPoint {
  pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovedRect {
  to: DirtyRect,
  from: MovedPoint,
}

impl MovedRect {
  pub fn new(to: DirtyRect, from: MovedPoint) -> Self {
    Self { to, from }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrameFormat {
  B8G8R8A8,
}

pub trait Display<'buf> {
  type Frame: Frame<'buf>;

  fn width(&self) -> Result<usize, DisplayError>;
  fn height(&self) -> Result<usize, DisplayError>;
  fn frame(&'buf mut self) -> Result<Self::Frame, FrameError>;
}

pub trait DisplayDriver<'buf> {
  type Display: 'static + Display<'buf> + Sized;

  fn name(&self) -> &'static str;

  fn all(&self) -> Result<Vec<Self::Display>, DisplayError>;
  fn primary(&self) -> Result<Option<Self::Display>, DisplayError>;
}
