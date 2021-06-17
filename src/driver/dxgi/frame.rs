use crate::{
  bindings::Windows::Win32::{
    Foundation::RECT,
    Graphics::Dxgi::{IDXGIOutputDuplication, DXGI_OUTDUPL_MOVE_RECT},
  },
  driver::dx11::frame::D3D11TextureFrameData,
  DirtyRect, Frame, FrameFormat, MovedPoint, MovedRect,
};
use std::{borrow::Cow, cmp::min};

#[derive(Debug, Clone)]
pub struct DxgiFrame<'a> {
  data: DxgiFrameData<'a>,
  dirty: Option<Vec<DirtyRect>>,
  duplication: &'a IDXGIOutputDuplication,
}

impl<'a> DxgiFrame<'a> {
  pub fn new<D>(data: D, duplication: &'a IDXGIOutputDuplication) -> Self
  where
    D: Into<DxgiFrameData<'a>>,
  {
    Self {
      data: data.into(),
      dirty: None,
      duplication,
    }
  }

  /// Get reference to underlying data
  pub const fn data(&self) -> &DxgiFrameData<'a> {
    &self.data
  }

  /// Get rectangles where pixels have changed since last frame
  pub fn dirty(&self) -> Vec<DirtyRect> {
    unsafe { self.get_dirty_rects() }
  }

  /// Get rectangles where pixels have moved since last frame
  pub fn moved(&self) -> Vec<MovedRect> {
    unsafe { self.get_moved_rects() }
  }

  /// Get pixel format of underlying data
  ///
  /// # Notes
  /// Per the Microsoft DesktopDuplication API documentation the format of the desktop
  /// image is always `DXGI_FORMAT_B8G8R8A8_UNORM` which translates to `B8G8R8A8`.
  ///
  /// https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api#updating-the-desktop-image-data
  pub const fn format(&self) -> FrameFormat {
    FrameFormat::B8G8R8A8
  }

  /// Get pixel data
  ///
  /// # Notes
  /// When frame data is [`DxgiFrameData::DirectX`] texture is copied to CPU memory and
  /// returned.  No caching occurs so, if you plan on using this multiple times you should
  /// probably cache the result yourself.  
  pub fn as_bytes(&self) -> anyhow::Result<Cow<'a, [u8]>> {
    match &self.data {
      DxgiFrameData::Memory(buf) => Ok(Cow::from(*buf)),
      DxgiFrameData::DirectX(texture) => Ok(Cow::from(texture.get_bytes()?)),
    }
  }

  /// Convert into underlying data
  pub fn into_data(self) -> DxgiFrameData<'a> {
    self.data
  }

  /// Gets dirty rectangles from [`IDXGIOutputDuplication`] while ignoring errors and doing
  /// best effort minimizing amount of memory while allowing further growth when needed.
  ///
  /// At some point I may consider caching [`RECT`] buffer and translated [`FrameRect`]
  /// items in [`DxgiFrame`] but, for the time being I'll let the end user decide where and
  /// how data is stored (with the exception of the initial allocations ofc)
  unsafe fn get_dirty_rects(&self) -> Vec<DirtyRect> {
    // Default rectangle buffer size (comes out to 2KB)
    const RECT_BUF_LEN: usize = 16;
    // Maximum rectangle buffer size (comes out to ~1MB)
    const RECT_BUF_MAX_LEN: usize = 7000 - RECT_BUF_LEN;

    let mut dirty = vec![RECT::default(); RECT_BUF_LEN];
    let mut dirty_len = 0;
    let _ = self.duplication.GetFrameDirtyRects(
      dirty.len() as _,
      dirty.as_mut_ptr(),
      &mut dirty_len,
    );

    let more = (dirty_len as usize).saturating_sub(dirty.len());
    let more = min(RECT_BUF_MAX_LEN, more);

    // `RECT_LEN` rectangles is not enough, try extending dirty
    if more > 0 {
      dirty.extend_from_slice(&vec![RECT::default(); more]);

      let _ = self.duplication.GetFrameDirtyRects(
        dirty.len() as _,
        dirty.as_mut_ptr(),
        &mut dirty_len,
      );
    }

    // I would _love_ if rust/llvm would optimize this away into a transparent type rather
    // than looping over a structure and mapping it into a structure that looks exactly the
    // same. I know Quartz, x11, and Wayland will have different definitions so we need a
    // generic type that will handle this and I _really_ don't want to add another nested
    // type definition to the trait tree for [`Frame`].
    dirty
      .into_iter()
      .take(dirty_len as usize)
      .map(|rect| DirtyRect::new(rect.top, rect.right, rect.bottom, rect.left))
      .collect()
  }

  unsafe fn get_moved_rects(&self) -> Vec<MovedRect> {
    // Default rectangle buffer size (comes out to 2KB)
    const RECT_BUF_LEN: usize = 16;
    // Maximum rectangle buffer size (comes out to ~1MB)
    const RECT_BUF_MAX_LEN: usize = 7000 - RECT_BUF_LEN;

    let mut moved = vec![DXGI_OUTDUPL_MOVE_RECT::default(); RECT_BUF_LEN];
    let mut moved_len = 0;
    let _ = self.duplication.GetFrameMoveRects(
      moved.len() as _,
      moved.as_mut_ptr(),
      &mut moved_len,
    );

    let more = (moved_len as usize).saturating_sub(moved.len());
    let more = min(RECT_BUF_MAX_LEN, more);

    // `RECT_LEN` rectangles is not enough, try extending dirty
    if more > 0 {
      moved.extend_from_slice(&vec![DXGI_OUTDUPL_MOVE_RECT::default(); more]);

      let _ = self.duplication.GetFrameMoveRects(
        moved.len() as _,
        moved.as_mut_ptr(),
        &mut moved_len,
      );
    }

    // I would _love_ if rust/llvm would optimize this away into a transparent type rather
    // than looping over a structure and mapping it into a structure that looks exactly the
    // same. I know Quartz, x11, and Wayland will have different definitions so we need a
    // generic type that will handle this and I _really_ don't want to add another nested
    // type definition to the trait tree for [`Frame`].
    moved
      .into_iter()
      .take(moved_len as usize)
      .map(|moved| {
        MovedRect::new(
          DirtyRect::new(
            moved.DestinationRect.top,
            moved.DestinationRect.right,
            moved.DestinationRect.bottom,
            moved.DestinationRect.left,
          ),
          MovedPoint::new(moved.SourcePoint.x, moved.SourcePoint.y),
        )
      })
      .collect()
  }
}

impl<'frame> Frame<'frame> for DxgiFrame<'frame> {
  fn dirty(&self) -> Vec<DirtyRect> {
    self.dirty()
  }

  fn moved(&self) -> Vec<MovedRect> {
    self.moved()
  }

  fn format(&self) -> FrameFormat {
    self.format()
  }

  fn as_bytes(&self) -> anyhow::Result<Cow<'frame, [u8]>> {
    self.as_bytes()
  }
}

#[derive(Debug, Clone)]
pub enum DxgiFrameData<'frame> {
  Memory(&'frame [u8]),
  DirectX(D3D11TextureFrameData<'frame>),
}

impl<'frame> From<&'frame [u8]> for DxgiFrameData<'frame> {
  fn from(data: &'frame [u8]) -> Self {
    Self::Memory(data)
  }
}

impl<'frame> From<D3D11TextureFrameData<'frame>> for DxgiFrameData<'frame> {
  fn from(data: D3D11TextureFrameData<'frame>) -> Self {
    Self::DirectX(data)
  }
}
