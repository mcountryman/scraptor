use crate::bindings::Windows::Win32::Graphics::{
  Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_CPU_ACCESS_READ,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
  },
  Dxgi::{IDXGISurface, DXGI_MAPPED_RECT, DXGI_MAP_READ, DXGI_RESOURCE_PRIORITY_MAXIMUM},
};
use std::slice;
use windows::Interface;

#[derive(Debug, Clone)]
pub struct Dx11FrameData<'frame> {
  device: &'frame ID3D11Device,
  context: &'frame ID3D11DeviceContext,
  texture: ID3D11Texture2D,
}

impl<'frame> Dx11FrameData<'frame> {
  pub fn new(
    device: &'frame ID3D11Device,
    context: &'frame ID3D11DeviceContext,
    texture: ID3D11Texture2D,
  ) -> Self {
    Self {
      device,
      context,
      texture,
    }
  }

  pub fn get_bytes(&self) -> anyhow::Result<Vec<u8>> {
    let mut rect = DXGI_MAPPED_RECT::default();
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    let data = unsafe {
      self.texture.GetDesc(&mut desc);
      self.get_surface()?.Map(&mut rect, DXGI_MAP_READ).ok()?;

      let len = desc.Height as usize * rect.Pitch as usize;
      let data = rect.pBits;

      slice::from_raw_parts(data, len).to_vec()
    };

    Ok(data)
  }

  unsafe fn get_surface(&self) -> anyhow::Result<IDXGISurface> {
    let mut texture_desc = D3D11_TEXTURE2D_DESC::default();

    self.texture.GetDesc(&mut texture_desc);
    texture_desc.Usage = D3D11_USAGE_STAGING;
    texture_desc.BindFlags = 0.into();
    texture_desc.MiscFlags = 0.into();
    texture_desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;

    let mut readable = None;

    self
      .device
      .CreateTexture2D(&texture_desc, std::ptr::null(), &mut readable)
      .ok()?;

    let readable = match readable {
      Some(readable) => readable,
      None => anyhow::bail!("Failed to create texture, texture is `None`"),
    };

    readable.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM.0);

    self.context.CopyResource(&readable, &self.texture);

    Ok(readable.cast()?)
  }
}

// let texture: ID3D11Texture2D = resource.cast()?;
// let mut texture_desc = D3D11_TEXTURE2D_DESC::default();

// texture.GetDesc(&mut texture_desc);
// texture_desc.Usage = D3D11_USAGE_STAGING;
// texture_desc.BindFlags = 0.into();
// texture_desc.MiscFlags = 0.into();
// texture_desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;

// let mut readable = None;

// device
//   .CreateTexture2D(&texture_desc, std::ptr::null(), &mut readable)
//   .ok()?;

// let readable = match readable {
//   Some(readable) => readable,
//   None => anyhow::bail!("Failed to create texture, texture is `None`"),
// };

// readable.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM.0);

// let readable: IDXGISurface = readable.cast()?;
