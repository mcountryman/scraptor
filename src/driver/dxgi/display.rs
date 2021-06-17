use super::{capture::DxgiDisplayCapturer, frame::DxgiFrame};
use crate::{
  bindings::Windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1, IDXGIOutput1, DXGI_ERROR_NOT_FOUND,
    DXGI_OUTPUT_DESC,
  },
  errors::{DisplayError, FrameError},
  Display,
};
use std::{hint::unreachable_unchecked, time::Duration};
use windows::Interface;

#[derive(Debug, Clone)]
pub struct DxgiDisplay {
  pub(super) desc: DXGI_OUTPUT_DESC,
  pub(super) output: IDXGIOutput1,
  pub(super) adapter: IDXGIAdapter1,
  pub(super) capturer: Option<DxgiDisplayCapturer>,
}

impl DxgiDisplay {
  pub fn name(&self) -> String {
    String::from_utf16_lossy(&self.desc.DeviceName)
  }

  pub const fn width(&self) -> usize {
    (self.desc.DesktopCoordinates.right - self.desc.DesktopCoordinates.left) as usize
  }

  pub const fn height(&self) -> usize {
    (self.desc.DesktopCoordinates.bottom - self.desc.DesktopCoordinates.top) as usize
  }

  unsafe fn capturer_mut<'a, 'b: 'a>(
    &'b mut self,
  ) -> Result<&'a mut DxgiDisplayCapturer, FrameError> {
    if self.capturer.is_none() {
      self.capturer = Some(DxgiDisplayCapturer::new(self).unwrap());
    }

    match &mut self.capturer {
      Some(capturer) => Ok(capturer),
      // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
      // variant in the code above.
      None => unreachable_unchecked(),
    }
  }
}

impl<'frame> Display<'frame> for DxgiDisplay {
  type Frame = DxgiFrame<'frame>;

  fn width(&self) -> Result<usize, DisplayError> {
    Ok(self.width())
  }

  fn height(&self) -> Result<usize, DisplayError> {
    Ok(self.height())
  }

  fn frame(&'frame mut self) -> Result<Self::Frame, FrameError> {
    // ~124fps to give windows a little time to prepare a frame for us.
    const FPS_124: u64 = 8;

    Ok(unsafe {
      self
        .capturer_mut()?
        .get_frame(Duration::from_millis(FPS_124))?
    })
  }
}

#[derive(Debug, Clone)]
pub struct DxgiDisplays {
  factory: IDXGIFactory1,
  adapter: Option<IDXGIAdapter1>,
  adapter_idx: u32,
  display_idx: u32,
}

impl DxgiDisplays {
  pub fn new() -> windows::Result<Self> {
    Ok(Self {
      factory: unsafe { CreateDXGIFactory1()? },
      adapter: None,
      adapter_idx: 0,
      display_idx: 0,
    })
  }

  unsafe fn next_display(&mut self) -> windows::Result<Option<DxgiDisplay>> {
    // Read in adapter if not already read
    if self.adapter.is_none() {
      let result = self
        .factory
        .EnumAdapters1(self.adapter_idx, &mut self.adapter);

      // If display adapter is not found, return `None`
      if result == DXGI_ERROR_NOT_FOUND {
        return Ok(None);
      }

      result.ok()?;
    }

    // Check if adapter is `None`
    match &self.adapter {
      None => Ok(None),
      // We have an adapter, now try reading outputs
      Some(adapter) => {
        // Read in output
        let mut output = None;
        let result = adapter.EnumOutputs(self.display_idx, &mut output);

        // If display output is not found, return `None`
        if result == DXGI_ERROR_NOT_FOUND {
          return Ok(None);
        }

        result.ok()?;

        match output {
          None => {
            // No more outputs for this adapter, increment adapter_idx, reset display_idx
            // for next adapter, and set adapter to `None` so we can load next iter
            self.adapter = None;
            self.adapter_idx += 1;
            self.display_idx = 0;

            self.next_display()
          }
          Some(output) => {
            // Read in output details
            let mut desc = DXGI_OUTPUT_DESC::default();
            output.GetDesc(&mut desc).ok()?;

            // Move to next display
            self.display_idx += 1;

            Ok(Some(DxgiDisplay {
              desc,
              output: output.cast()?,
              adapter: adapter.clone(),
              capturer: None,
            }))
          }
        }
      }
    }
  }
}

impl Iterator for DxgiDisplays {
  type Item = windows::Result<DxgiDisplay>;

  fn next(&mut self) -> Option<Self::Item> {
    unsafe { self.next_display() }.transpose()
  }
}

#[cfg(test)]
mod tests {
  use super::DxgiDisplays;

  #[test]
  fn test_next_dxgi_display() {
    let displays = DxgiDisplays::new().unwrap();
    for display in displays {
      let display = display.unwrap();

      println!(
        "`{}` w:{}, h:{}",
        display.name(),
        display.width(),
        display.height()
      );
    }
  }
}
