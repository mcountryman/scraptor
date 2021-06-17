use super::{display::DxgiDisplay, errors::FrameError};
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
  driver::dx11::frame::D3D11TextureFrame,
  frame::{Frame, PixelFormat},
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

    let device = device.expect("Got empty device without error");
    let context = context.expect("Got empty device context without error");

    display
      .output
      .DuplicateOutput(device.clone(), &mut duplication)
      .ok()?;

    let duplication = duplication.expect("Got empty duplication without error");
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
  pub unsafe fn get_frame(&mut self, timeout: Duration) -> Result<Frame<'_>, FrameError> {
    let mut frame = DXGI_OUTDUPL_FRAME_INFO::default();
    let mut resource = None;

    if self.desc.DesktopImageInSystemMemory.as_bool() {
      self.duplication.UnMapDesktopSurface().ok()?;
    }

    // Release frame and ignore error
    let _ = self.duplication.ReleaseFrame();

    match self.duplication.AcquireNextFrame(
      timeout.as_millis() as u32,
      &mut frame,
      &mut resource,
    ) {
      result if result.0 == DXGI_ERROR_WAIT_TIMEOUT.0 => {
        return Err(FrameError::WouldBlock)
      }
      result => result.ok()?,
    };

    Ok(if self.desc.DesktopImageInSystemMemory.as_bool() {
      self.duplication.MapDesktopSurface(&mut self.rect).ok()?;

      let buf = self.rect.pBits;
      let len = (self.desc.ModeDesc.Height * self.rect.Pitch as u32) as usize;
      let buf = slice::from_raw_parts(buf, len);

      Frame::Memory {
        fmt: PixelFormat::Bgra,
        buf,
      }
    } else if let Some(resource) = resource {
      D3D11TextureFrame::new(&self.device, &self.context, resource.cast()?).into()
    } else {
      todo!("Failed to read IDXGIResource")
    })
  }
}

#[cfg(test)]
mod tests {
  use super::DxgiDisplayCapturer;
  use crate::driver::dxgi::display::DxgiDisplays;
  use std::time::Duration;

  #[test]
  fn test_get_frame() {
    unsafe {
      let mut displays = DxgiDisplays::new().unwrap();
      let display = displays.next().unwrap().unwrap();
      let mut capturer = DxgiDisplayCapturer::new(&display).unwrap();

      for _ in 0..10 {
        capturer
          .get_frame(Duration::from_millis(50))
          .unwrap()
          .as_bytes()
          .unwrap();
      }
    }
  }
}
