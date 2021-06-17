use super::{display::DxgiDisplay, errors::FrameError, frame::DxgiFrame};
use crate::{
  bindings::Windows::Win32::{
    Foundation::HINSTANCE,
    Graphics::{
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_DEBUG,
        D3D11_SDK_VERSION, D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_9_1,
      },
      Dxgi::{
        IDXGIOutputDuplication, DXGI_ERROR_WAIT_TIMEOUT, DXGI_MAPPED_RECT,
        DXGI_OUTDUPL_DESC, DXGI_OUTDUPL_FRAME_INFO,
      },
    },
  },
  driver::dx11::frame::D3D11TextureFrameData,
};
use std::{slice, time::Duration};
use windows::Interface;

#[derive(Debug, Clone)]
pub struct DxgiDisplayCapturer {
  rect: DXGI_MAPPED_RECT,
  desc: DXGI_OUTDUPL_DESC,
  device: ID3D11Device,
  context: ID3D11DeviceContext,
  duplication: IDXGIOutputDuplication,
  has_frame: bool,
}

impl DxgiDisplayCapturer {
  /// Create [`DxgiDisplayCapturer`] for supplied display.
  ///
  /// # Arguments
  /// * `display` - The display to create capturer for.
  ///
  /// # Safety
  /// Heavy use of unsafe calls to DirectX 11 and DXGI.
  pub unsafe fn new(display: &DxgiDisplay) -> Result<Self, FrameError> {
    let mut level = D3D_FEATURE_LEVEL_9_1;
    let mut device = None;
    let mut context = None;
    let mut duplication = None;

    D3D11CreateDevice(
      display.adapter.clone(),
      D3D_DRIVER_TYPE_UNKNOWN,
      HINSTANCE::NULL,
      D3D11_CREATE_DEVICE_DEBUG,
      std::ptr::null_mut(),
      0,
      D3D11_SDK_VERSION,
      &mut device,
      &mut level,
      &mut context,
    )
    .ok()?;

    let device = device.ok_or(FrameError::None)?;
    let context = context.ok_or(FrameError::None)?;

    display
      .output
      .DuplicateOutput(device.clone(), &mut duplication)
      .ok()?;

    let duplication = duplication.ok_or(FrameError::None)?;
    let mut desc = DXGI_OUTDUPL_DESC::default();

    duplication.GetDesc(&mut desc);

    // Pre-load frame to allow for `ReleaseFrame` on subsequent calls to `get_frame`
    let mut frame = DXGI_OUTDUPL_FRAME_INFO::default();
    let mut resource = None;

    duplication
      .AcquireNextFrame(0, &mut frame, &mut resource)
      .ok()
      .map_err(FrameError::AcquireFrame)?;

    Ok(Self {
      rect: DXGI_MAPPED_RECT::default(),
      desc,
      device,
      context,
      duplication,
      has_frame: true,
    })
  }

  /// Read next from from DXGI.
  ///
  /// # Arguments
  /// * `timeout` - The amount of time that this method waits for a new frame before it
  /// returns to the caller.
  ///
  /// # Safety
  /// Heavy use of unsafe calls to DirectX 11 and DXGI.
  pub unsafe fn get_frame<'a, 'b: 'a>(
    &'b mut self,
    timeout: Duration,
  ) -> Result<DxgiFrame<'a>, FrameError> {
    let mut frame = DXGI_OUTDUPL_FRAME_INFO::default();
    let mut resource = None;

    // Check if frame was already release before making calls
    if self.has_frame {
      // Release frame memory and ignore error
      if self.desc.DesktopImageInSystemMemory.as_bool() {
        let _ = self.duplication.UnMapDesktopSurface();
      }

      // Release frame and ignore error
      let _ = self.duplication.ReleaseFrame();

      self.has_frame = false;
    }

    match self.duplication.AcquireNextFrame(
      timeout.as_millis() as u32,
      &mut frame,
      &mut resource,
    ) {
      // If `AcquireNextFrame` timeout hits return `WouldBlock` error
      result if result.0 == DXGI_ERROR_WAIT_TIMEOUT.0 => {
        return Err(FrameError::WouldBlock)
      }
      result => result.ok()?,
    };

    // Indicate a frame needs to be released before calling `AcquireNextFrame`.
    self.has_frame = true;

    // Frame is already in system memory, map to `DXGI_MAPPED_RECT` and cast to slice
    if self.desc.DesktopImageInSystemMemory.as_bool() {
      self.duplication.MapDesktopSurface(&mut self.rect).ok()?;

      let buf = self.rect.pBits;
      let len = (self.desc.ModeDesc.Height * self.rect.Pitch as u32) as usize;
      let buf = slice::from_raw_parts(buf, len);

      return Ok(DxgiFrame::new(buf, &self.duplication));
    }

    if let Some(resource) = resource {
      let device = &self.device;
      let context = &self.context;
      let texture = resource.cast()?;
      let texture = D3D11TextureFrameData::new(device, context, texture);

      Ok(DxgiFrame::new(texture, &self.duplication))
    } else {
      Err(FrameError::None)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::DxgiDisplayCapturer;
  use crate::{driver::dxgi::display::DxgiDisplays, Frame};
  use std::time::Duration;

  #[test]
  fn test_get_frame() {
    unsafe {
      let mut displays = DxgiDisplays::new().unwrap();
      let display = displays.next().unwrap().unwrap();
      let mut capturer = DxgiDisplayCapturer::new(&display).unwrap();

      for _ in 0..10 {
        let frame = capturer.get_frame(Duration::from_millis(16)).unwrap();

        let frame_buf = frame.as_bytes().unwrap();
        let _ = frame.dirty();

        assert!(frame_buf.len() > 0);
      }
    }
  }
}
