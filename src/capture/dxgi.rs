use image::RgbaImage;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_UNKNOWN,
    D3D_FEATURE_LEVEL_11_0,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, D3D11_CPU_ACCESS_READ,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE,
    D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_STAGING, ID3D11Device,
    ID3D11DeviceContext, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_B8G8R8A8_UNORM,
    DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1,
    IDXGIOutput1, IDXGIOutputDuplication,
    DXGI_OUTDUPL_FRAME_INFO,
};
use windows::core::Interface;

pub struct DxgiCapture {
    device: ID3D11Device,
    ctx: ID3D11DeviceContext,
    dupl: IDXGIOutputDuplication,
    staging: ID3D11Texture2D,
    width: u32,
    height: u32,
}

impl DxgiCapture {
    pub fn new() -> Result<Self, String> {
        let factory: IDXGIFactory1 = unsafe {
            CreateDXGIFactory1()
        }
        .map_err(|e| {
            format!("CreateDXGIFactory1: {e}")
        })?;

        let adapter = unsafe {
            factory.EnumAdapters1(0)
        }
        .map_err(|e| {
            format!("EnumAdapters1: {e}")
        })?;

        let mut device = None;
        unsafe {
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                None,
            )
        }
        .map_err(|e| {
            format!("D3D11CreateDevice: {e}")
        })?;
        let device = device.unwrap();

        let output = unsafe {
            adapter.EnumOutputs(0)
        }
        .map_err(|e| {
            format!("EnumOutputs: {e}")
        })?;
        let output1: IDXGIOutput1 = output
            .cast()
            .map_err(|e| {
                format!("cast Output1: {e}")
            })?;

        let dupl = unsafe {
            output1.DuplicateOutput(&device)
        }
        .map_err(|e| {
            format!("DuplicateOutput: {e}")
        })?;

        let dupl_desc = unsafe {
            dupl.GetDesc()
        };
        let width =
            dupl_desc.ModeDesc.Width;
        let height =
            dupl_desc.ModeDesc.Height;

        let staging_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: Default::default(),
            CPUAccessFlags: D3D11_CPU_ACCESS_READ
                .0 as u32,
            MiscFlags: Default::default(),
        };

        let mut staging = None;
        unsafe {
            device.CreateTexture2D(
                &staging_desc,
                None,
                Some(&mut staging),
            )
        }
        .map_err(|e| {
            format!("CreateTexture2D: {e}")
        })?;
        let staging = staging.unwrap();

        let ctx = unsafe {
            device.GetImmediateContext()
        }
        .map_err(|e| {
            format!("GetContext: {e}")
        })?;

        log::info!(
            "DXGI capture initialized: {}x{}",
            width, height,
        );

        Ok(Self {
            device,
            ctx,
            dupl,
            staging,
            width,
            height,
        })
    }

    /// Acquire frame from DXGI without
    /// GPU→CPU copy. Returns capture timestamp.
    pub fn acquire_frame(
        &self,
    ) -> Result<(), String> {
        let mut frame_info =
            DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource = None;
        unsafe {
            self.dupl.AcquireNextFrame(
                1000,
                &mut frame_info,
                &mut resource,
            )
        }
        .map_err(|e| {
            format!("AcquireNextFrame: {e}")
        })?;

        let resource = resource.unwrap();
        let tex: ID3D11Texture2D = resource
            .cast()
            .map_err(|e| {
                format!("cast tex: {e}")
            })?;

        unsafe {
            self.ctx.CopyResource(
                &self.staging, &tex,
            );
        }
        Ok(())
    }

    pub fn screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Read pixels from staging texture directly
    /// into a caller-provided buffer (e.g. DIB
    /// bits). Zero intermediate allocation.
    /// `dst` must be at least w*h*4 bytes.
    pub fn read_pixels_into(
        &self,
        dst: *mut u8,
        dst_len: usize,
    ) -> Result<(u32, u32), String> {
        let mut mapped =
            D3D11_MAPPED_SUBRESOURCE::default();
        unsafe {
            self.ctx.Map(
                &self.staging,
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped),
            )
        }
        .map_err(|e| format!("Map: {e}"))?;

        let w = self.width;
        let h = self.height;
        let row_pitch =
            mapped.RowPitch as usize;
        let row_bytes = (w * 4) as usize;
        let total = (w * h * 4) as usize;
        assert!(dst_len >= total);

        unsafe {
            let src =
                mapped.pData as *const u8;
            if row_pitch == row_bytes {
                std::ptr::copy_nonoverlapping(
                    src, dst, total,
                );
            } else {
                for y in 0..h as usize {
                    let src_row =
                        src.add(y * row_pitch);
                    let dst_row =
                        dst.add(y * row_bytes);
                    std::ptr::copy_nonoverlapping(
                        src_row,
                        dst_row,
                        row_bytes,
                    );
                }
            }
        }

        unsafe {
            self.ctx.Unmap(&self.staging, 0);
        }
        unsafe { self.dupl.ReleaseFrame() }
            .ok();

        Ok((w, h))
    }

    /// Read pixels from staging texture
    /// into a Vec. Call after acquire_frame.
    pub fn read_pixels(
        &self,
    ) -> Result<(Vec<u8>, u32, u32), String> {
        let (buf, w, h) = self.read_staging()?;
        unsafe { self.dupl.ReleaseFrame() }.ok();
        Ok((buf, w, h))
    }

    /// Read staging texture into Vec without
    /// releasing the DXGI frame. Staging data
    /// persists even after ReleaseFrame, so
    /// this also works after read_pixels_into.
    pub fn read_staging(
        &self,
    ) -> Result<(Vec<u8>, u32, u32), String> {
        let mut mapped =
            D3D11_MAPPED_SUBRESOURCE::default();
        unsafe {
            self.ctx.Map(
                &self.staging,
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped),
            )
        }
        .map_err(|e| format!("Map: {e}"))?;

        let w = self.width;
        let h = self.height;
        let row_pitch =
            mapped.RowPitch as usize;
        let row_bytes = (w * 4) as usize;
        let total_bytes =
            (w * h * 4) as usize;
        let mut buf = vec![0u8; total_bytes];

        unsafe {
            let src =
                mapped.pData as *const u8;
            if row_pitch == row_bytes {
                std::ptr::copy_nonoverlapping(
                    src,
                    buf.as_mut_ptr(),
                    total_bytes,
                );
            } else {
                for y in 0..h as usize {
                    let src_row =
                        src.add(y * row_pitch);
                    let dst = &mut buf
                        [y * row_bytes..]
                        [..row_bytes];
                    std::ptr::copy_nonoverlapping(
                        src_row,
                        dst.as_mut_ptr(),
                        row_bytes,
                    );
                }
            }
        }

        unsafe {
            self.ctx.Unmap(&self.staging, 0);
        }

        Ok((buf, w, h))
    }

    pub fn bgra_to_rgba(buf: &mut [u8]) {
        let len = buf.len();
        let mut i = 0;
        while i + 8 <= len {
            buf.swap(i, i + 2);
            buf.swap(i + 4, i + 6);
            i += 8;
        }
        if i + 4 <= len {
            buf.swap(i, i + 2);
        }
    }

    pub fn capture(
        &self,
    ) -> Result<RgbaImage, String> {
        self.acquire_frame()?;
        let (mut buf, w, h) =
            self.read_pixels()?;
        Self::bgra_to_rgba(&mut buf);
        RgbaImage::from_raw(w, h, buf)
            .ok_or_else(|| {
                "Failed to create image".into()
            })
    }
}
