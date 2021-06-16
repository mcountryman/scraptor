use crate::{
  errors::{DisplayError, FrameError},
  DisplayDriver, DisplayHandle, Frame,
};

mod bindings {
  windows::include_bindings!();
}

use bindings::Windows::Win32::Graphics::Dxgi::*;

#[derive(Debug, Clone)]
pub struct Dxgi;

#[derive(Debug, Clone)]
pub struct DxgiDisplay {
  desc: DXGI_OUTPUT_DESC,
  output: IDXGIOutput,
  adapter: IDXGIAdapter1,
}

impl DxgiDisplay {
  pub fn name(&self) -> String {
    String::from_utf16_lossy(&self.desc.DeviceName)
  }

  pub fn width(&self) -> usize {
    (self.desc.DesktopCoordinates.right - self.desc.DesktopCoordinates.left) as usize
  }

  pub fn height(&self) -> usize {
    (self.desc.DesktopCoordinates.bottom - self.desc.DesktopCoordinates.top) as usize
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
              adapter: adapter.clone(),
              output,
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

impl Dxgi {}

impl DisplayDriver for Dxgi {
  fn name(&self) -> &'static str {
    "dxgi"
  }

  fn get_all(&self) -> Result<Vec<DisplayHandle>, DisplayError> {
    todo!()
  }

  fn get_primary(&self) -> Result<Option<DisplayHandle>, DisplayError> {
    todo!()
  }

  fn get_display_frame(&self, _: DisplayHandle) -> Result<Frame<'_>, FrameError> {
    todo!()
  }

  fn get_display_width(&self, _: DisplayHandle) -> Result<usize, DisplayError> {
    todo!()
  }

  fn get_display_height(&self, _: DisplayHandle) -> Result<usize, DisplayError> {
    todo!()
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

  #[test]
  fn test_get_all() {}
}
