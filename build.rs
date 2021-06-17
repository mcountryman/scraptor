fn main() {
  windows::build! {
    Windows::Win32::Graphics::Dxgi::*,
    Windows::Win32::Graphics::Direct3D11::*,
  };
}
