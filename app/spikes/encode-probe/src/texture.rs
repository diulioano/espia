//! Quadro NV12 gerado no processador (barras de cor, contador e clarão, como em `media.rs`) e a textura NV12 na
//! placa de vídeo que o codificador recebe (T010).
//!
//! Na sonda se prova a LIGAÇÃO (textura no mesmo dispositivo Direct3D 11 do codificador), não a captura: o quadro
//! nasce no processador e sobe para uma textura NV12. No app, a captura já entrega essa textura pronta (R3).
//!
//! Nada aqui captura tela, câmera ou microfone.

use std::time::Duration;

use windows::core::Interface;
use windows::Win32::Foundation::{HMODULE, LUID};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Multithread, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
    D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_WRITE, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_NV12, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter, IDXGIDevice, IDXGIFactory4};

// ---------------------------------------------------------------- dispositivo Direct3D 11

/// Cria um dispositivo Direct3D 11 próprio (suporte a vídeo, proteção para várias threads ligada).
/// Com `luid`, usa a placa que o codificador informou (computador com duas placas); sem, a padrão.
pub fn create_device(luid: Option<LUID>) -> Result<ID3D11Device, String> {
    let adapter: Option<IDXGIAdapter> = match luid {
        Some(l) => unsafe {
            let factory: IDXGIFactory4 = CreateDXGIFactory1().map_err(|e| format!("CreateDXGIFactory1: {e}"))?;
            Some(factory.EnumAdapterByLuid::<IDXGIAdapter>(l).map_err(|e| format!("EnumAdapterByLuid: {e}"))?)
        },
        None => None,
    };
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT;
    let mut device = None;
    unsafe {
        D3D11CreateDevice(
            adapter.as_ref(),
            if adapter.is_some() { D3D_DRIVER_TYPE_UNKNOWN } else { D3D_DRIVER_TYPE_HARDWARE },
            HMODULE::default(),
            flags,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )
        .map_err(|e| format!("D3D11CreateDevice: {e}"))?;
    }
    let device = device.ok_or("D3D11CreateDevice não devolveu dispositivo")?;
    // O codificador (e, no app, a captura) usam o mesmo dispositivo em outras threads.
    if let Ok(mt) = device.cast::<ID3D11Multithread>() {
        unsafe {
            let _ = mt.SetMultithreadProtected(true);
        }
    }
    Ok(device)
}

/// Nome da placa de vídeo do dispositivo (para o log).
pub fn adapter_name(device: &ID3D11Device) -> String {
    unsafe {
        let Ok(dxgi) = device.cast::<IDXGIDevice>() else { return "?".into() };
        let Ok(adapter) = dxgi.GetAdapter() else { return "?".into() };
        match adapter.GetDesc() {
            Ok(d) => {
                let n = d.Description.iter().position(|c| *c == 0).unwrap_or(d.Description.len());
                String::from_utf16_lossy(&d.Description[..n])
            }
            Err(_) => "?".into(),
        }
    }
}

// ---------------------------------------------------------------- texturas NV12

/// Anel de texturas NV12 na placa. O codificador pode segurar uma textura até devolver o quadro comprimido,
/// por isso há várias e cada quadro vai para a seguinte.
pub struct TextureRing {
    ctx: ID3D11DeviceContext,
    staging: ID3D11Texture2D,
    ring: Vec<ID3D11Texture2D>,
    next: usize,
    w: u32,
    h: u32,
}

fn nv12_texture(device: &ID3D11Device, w: u32, h: u32, staging: bool) -> Result<ID3D11Texture2D, String> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: w,
        Height: h,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_NV12,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: if staging { D3D11_USAGE_STAGING } else { D3D11_USAGE_DEFAULT },
        // Como a saída do conversor de vídeo da captura de hoje (src-tauri/src/capture/video.rs): NV12 com
        // RENDER_TARGET.
        BindFlags: if staging { 0 } else { D3D11_BIND_RENDER_TARGET.0 as u32 },
        CPUAccessFlags: if staging { D3D11_CPU_ACCESS_WRITE.0 as u32 } else { 0 },
        MiscFlags: 0,
    };
    let mut tex = None;
    unsafe {
        device.CreateTexture2D(&desc, None, Some(&mut tex)).map_err(|e| format!("CreateTexture2D NV12 {w}x{h}: {e}"))?;
    }
    tex.ok_or_else(|| "CreateTexture2D não devolveu textura".to_string())
}

impl TextureRing {
    pub fn new(device: &ID3D11Device, w: u32, h: u32, count: usize) -> Result<Self, String> {
        let ctx = unsafe { device.GetImmediateContext().map_err(|e| format!("GetImmediateContext: {e}"))? };
        let staging = nv12_texture(device, w, h, true)?;
        let mut ring = Vec::with_capacity(count);
        for _ in 0..count.max(2) {
            ring.push(nv12_texture(device, w, h, false)?);
        }
        Ok(Self { ctx, staging, ring, next: 0, w, h })
    }

    /// Sobe um quadro NV12 compacto (Y w×h, depois UV intercalado w×h/2, sem folga entre linhas) para a próxima
    /// textura do anel e a devolve.
    pub fn upload(&mut self, nv12: &[u8]) -> Result<ID3D11Texture2D, String> {
        let (w, h) = (self.w as usize, self.h as usize);
        if nv12.len() < w * h * 3 / 2 {
            return Err(format!("quadro NV12 curto: {} bytes, esperado {}", nv12.len(), w * h * 3 / 2));
        }
        unsafe {
            let mut map = D3D11_MAPPED_SUBRESOURCE::default();
            self.ctx.Map(&self.staging, 0, D3D11_MAP_WRITE, 0, Some(&mut map)).map_err(|e| format!("Map da textura de apoio: {e}"))?;
            let pitch = map.RowPitch as usize;
            let dst = map.pData as *mut u8;
            // Mesma conta da leitura em video.rs: Y são h linhas; o UV vem logo depois, com o mesmo pitch.
            for row in 0..h {
                std::ptr::copy_nonoverlapping(nv12.as_ptr().add(row * w), dst.add(row * pitch), w);
            }
            let uv_dst = dst.add(pitch * h);
            let uv_src = nv12.as_ptr().add(w * h);
            for row in 0..h / 2 {
                std::ptr::copy_nonoverlapping(uv_src.add(row * w), uv_dst.add(row * pitch), w);
            }
            self.ctx.Unmap(&self.staging, 0);
            let tex = self.ring[self.next].clone();
            self.next = (self.next + 1) % self.ring.len();
            self.ctx.CopyResource(&tex, &self.staging);
            // Empurra o comando para a placa antes de entregar a textura ao codificador.
            self.ctx.Flush();
            Ok(tex)
        }
    }
}

// ---------------------------------------------------------------- gerador de quadros NV12

const FLASH_MS: u64 = 100;

/// Barras de cor (75% de cima), degradê (17%) e faixa do clarão (8% de baixo), em NV12 compacto. Por quadro:
/// contador binário de 16 bits no canto, barra branca andando e clarão de 1 s (como `media.rs`).
pub struct Nv12Generator {
    w: usize,
    h: usize,
    template: Vec<u8>,
    noise: bool,
    rng: u32,
}

impl Nv12Generator {
    /// `noise`: enche o quarto central da imagem de ruído novo a cada quadro, para o codificador ter o que gastar
    /// em taxa (sem isso, barras paradas comprimem a quase nada e a mudança de taxa não aparece no tamanho).
    pub fn new(w: u32, h: u32, noise: bool) -> Self {
        let (w, h) = (w as usize, h as usize);
        let mut t = vec![0u8; w * h * 3 / 2];
        // Barras SMPTE simplificadas (Y, U, V em BT.601 de faixa limitada), as mesmas de media.rs.
        const BARS: [(u8, u8, u8); 7] = [
            (180, 128, 128),
            (162, 44, 142),
            (131, 156, 44),
            (112, 72, 58),
            (84, 184, 198),
            (65, 100, 212),
            (35, 212, 114),
        ];
        let (y_plane, uv_plane) = t.split_at_mut(w * h);
        y_plane.fill(16);
        uv_plane.fill(128);
        let bars_h = h * 75 / 100;
        for row in 0..bars_h {
            for col in 0..w {
                y_plane[row * w + col] = BARS[col * 7 / w].0;
            }
        }
        for row in 0..bars_h / 2 {
            for c in 0..w / 2 {
                let b = BARS[c * 7 / (w / 2)];
                uv_plane[row * w + 2 * c] = b.1;
                uv_plane[row * w + 2 * c + 1] = b.2;
            }
        }
        let grad_end = h * 92 / 100;
        for row in bars_h..grad_end {
            for col in 0..w {
                y_plane[row * w + col] = (16 + col * 219 / w) as u8;
            }
        }
        Self { w, h, template: t, noise, rng: 0x1234_5678 }
    }

    /// Quadro número `n`, `elapsed` depois do começo (para o clarão de 1 s).
    pub fn frame(&mut self, n: u64, elapsed: Duration) -> Vec<u8> {
        let (w, h) = (self.w, self.h);
        let mut buf = self.template.clone();
        let y = &mut buf[..w * h];
        // Contador binário de 16 bits (quadrados) no canto superior esquerdo.
        let sq = 40usize.min(w / 40).max(4);
        for bit in 0..16usize {
            let val = if (n >> (15 - bit)) & 1 == 1 { 235 } else { 16 };
            let x0 = 20 + bit * (sq + 4);
            for row in 20..(20 + sq).min(h) {
                for col in x0..(x0 + sq).min(w) {
                    y[row * w + col] = val;
                }
            }
        }
        // Barra branca andando (16 px), nas barras de cor.
        if w > 32 {
            let bar_x = ((n * 12) % (w as u64 - 16)) as usize;
            for row in 0..h * 75 / 100 {
                for col in bar_x..bar_x + 16 {
                    y[row * w + col] = 235;
                }
            }
        }
        // Clarão de 1 s: faixa inferior branca nos primeiros 100 ms do segundo.
        if (elapsed.as_millis() as u64) % 1000 < FLASH_MS {
            for row in h * 93 / 100..h * 99 / 100 {
                for col in 0..w {
                    y[row * w + col] = 235;
                }
            }
        }
        if self.noise {
            let (r0, r1) = (h * 30 / 100, h * 70 / 100);
            let (c0, c1) = (w / 4, w * 3 / 4);
            let mut s = self.rng;
            for row in r0..r1 {
                for col in c0..c1 {
                    s ^= s << 13;
                    s ^= s >> 17;
                    s ^= s << 5;
                    y[row * w + col] = 16 + (s % 220) as u8;
                }
            }
            self.rng = s;
        }
        buf
    }
}
