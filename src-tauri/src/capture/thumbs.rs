// Miniaturas e ícones (research R2/R3): GDI para tirar a imagem, WIC (nativo do Windows) para virar
// JPEG/PNG sem crate de imagem. Tudo fica no PC: vira data: URL só para o seletor.

/// Maior tamanho que cabe em (max_w, max_h) mantendo a proporção. Nunca 0.
pub fn fit(w: i32, h: i32, max_w: i32, max_h: i32) -> (i32, i32) {
  if w <= 0 || h <= 0 {
    return (max_w, max_h);
  }
  let scale = (max_w as f64 / w as f64).min(max_h as f64 / h as f64).min(1.0);
  (((w as f64 * scale).round() as i32).max(1), ((h as f64 * scale).round() as i32).max(1))
}

/// Imagem toda preta (ou quase): janela que o PrintWindow não consegue desenhar.
pub fn is_black(bgra: &[u8]) -> bool {
  bgra.as_chunks::<4>().0.iter().step_by(7).all(|p| p[0] < 8 && p[1] < 8 && p[2] < 8)
}

pub const THUMB_W: i32 = 320;
pub const THUMB_H: i32 = 180;

#[cfg(windows)]
pub use imp::{icon_png, monitor_thumb, window_thumb};

#[cfg(windows)]
mod imp {
  use base64::{Engine, engine::general_purpose::STANDARD};
  use windows::{
    Win32::{
      Foundation::{HWND, RECT},
      Graphics::{
        Gdi::{
          BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection,
          DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, HALFTONE, HBITMAP, HDC, ReleaseDC, SRCCOPY, SelectObject,
          SetStretchBltMode, StretchBlt,
        },
        Imaging::{
          CLSID_WICImagingFactory, GUID_ContainerFormatJpeg, GUID_ContainerFormatPng, GUID_WICPixelFormat24bppBGR,
          GUID_WICPixelFormat32bppBGRA, IWICImagingFactory, WICBitmapEncoderNoCache,
        },
      },
      Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow},
      System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, IStream, STATFLAG_NONAME, STREAM_SEEK_SET},
      UI::WindowsAndMessaging::{DI_NORMAL, DrawIconEx, GetWindowRect, HICON, IsIconic},
    },
    core::GUID,
  };

  use super::{THUMB_H, THUMB_W, fit, is_black};
  use crate::capture::com_init;

  /// PW_RENDERFULLCONTENT: pega também o que é desenhado por DirectX.
  const PW_RENDERFULLCONTENT: PRINT_WINDOW_FLAGS = PRINT_WINDOW_FLAGS(2);

  /// DIB 32 bpp de cima para baixo; devolve o bitmap e os bytes BGRA.
  unsafe fn dib(dc: HDC, w: i32, h: i32) -> Option<(HBITMAP, *mut u8)> {
    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: -h,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };
    let mut bits = std::ptr::null_mut();
    let bmp = unsafe { CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) }.ok()?;
    (!bits.is_null()).then_some((bmp, bits as *mut u8))
  }

  /// Reduz `src_dc` (w×h) para caber em 320×180 e devolve os pixels BGRA.
  unsafe fn shrink(src_dc: HDC, x: i32, y: i32, w: i32, h: i32) -> Option<(i32, i32, Vec<u8>)> {
    let (tw, th) = fit(w, h, THUMB_W, THUMB_H);
    unsafe {
      let dc = CreateCompatibleDC(Some(src_dc));
      let (bmp, bits) = dib(dc, tw, th)?;
      let old = SelectObject(dc, bmp.into());
      SetStretchBltMode(dc, HALFTONE);
      let ok = StretchBlt(dc, 0, 0, tw, th, Some(src_dc), x, y, w, h, SRCCOPY).as_bool();
      let pixels = std::slice::from_raw_parts(bits, (tw * th * 4) as usize).to_vec();
      SelectObject(dc, old);
      let _ = DeleteObject(bmp.into());
      let _ = DeleteDC(dc);
      ok.then_some((tw, th, pixels))
    }
  }

  /// Miniatura de uma janela, ou None (minimizada, preta, falhou).
  pub fn window_thumb(hwnd: HWND) -> Option<String> {
    unsafe {
      if IsIconic(hwnd).as_bool() {
        return None;
      }
      let mut r = RECT::default();
      GetWindowRect(hwnd, &mut r).ok()?;
      let (w, h) = (r.right - r.left, r.bottom - r.top);
      if w <= 0 || h <= 0 {
        return None;
      }
      let screen = GetDC(None);
      let dc = CreateCompatibleDC(Some(screen));
      let bmp = CreateCompatibleBitmap(screen, w, h);
      let old = SelectObject(dc, bmp.into());
      let printed = PrintWindow(hwnd, dc, PW_RENDERFULLCONTENT).as_bool();
      let small = if printed { shrink(dc, 0, 0, w, h) } else { None };
      SelectObject(dc, old);
      let _ = DeleteObject(bmp.into());
      let _ = DeleteDC(dc);
      ReleaseDC(None, screen);
      let (tw, th, px) = small?;
      if is_black(&px) {
        return None;
      }
      encode(&px, tw, th, false).map(|b| format!("data:image/jpeg;base64,{}", STANDARD.encode(b)))
    }
  }

  /// Miniatura de um monitor (retângulo na área de trabalho virtual).
  pub fn monitor_thumb(r: RECT) -> Option<String> {
    unsafe {
      let screen = GetDC(None);
      let small = shrink(screen, r.left, r.top, r.right - r.left, r.bottom - r.top);
      ReleaseDC(None, screen);
      let (tw, th, px) = small?;
      encode(&px, tw, th, false).map(|b| format!("data:image/jpeg;base64,{}", STANDARD.encode(b)))
    }
  }

  /// Ícone 32×32 em PNG com transparência.
  pub fn icon_png(icon: HICON) -> Option<String> {
    const S: i32 = 32;
    unsafe {
      let screen = GetDC(None);
      let dc = CreateCompatibleDC(Some(screen));
      let out = (|| {
        let (bmp, bits) = dib(dc, S, S)?;
        let old = SelectObject(dc, bmp.into());
        let drawn = DrawIconEx(dc, 0, 0, icon, S, S, 0, None, DI_NORMAL).is_ok();
        let mut px = std::slice::from_raw_parts(bits, (S * S * 4) as usize).to_vec();
        SelectObject(dc, old);
        let _ = DeleteObject(bmp.into());
        if !drawn {
          return None;
        }
        // Ícone antigo sem alfa: o que foi desenhado fica opaco.
        if px.as_chunks::<4>().0.iter().all(|p| p[3] == 0) {
          for p in px.as_chunks_mut::<4>().0 {
            if p[0] | p[1] | p[2] != 0 {
              p[3] = 255;
            }
          }
        }
        encode(&px, S, S, true)
      })();
      let _ = DeleteDC(dc);
      ReleaseDC(None, screen);
      out.map(|b| format!("data:image/png;base64,{}", STANDARD.encode(b)))
    }
  }

  /// BGRA → JPEG (qualidade padrão do WIC) ou PNG.
  fn encode(bgra: &[u8], w: i32, h: i32, png: bool) -> Option<Vec<u8>> {
    let _com = com_init();
    unsafe {
      let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
      let stream: IStream = windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal(Default::default(), true).ok()?;
      let container: GUID = if png { GUID_ContainerFormatPng } else { GUID_ContainerFormatJpeg };
      let encoder = factory.CreateEncoder(&container, std::ptr::null()).ok()?;
      encoder.Initialize(&stream, WICBitmapEncoderNoCache).ok()?;
      let mut frame = None;
      encoder.CreateNewFrame(&mut frame, std::ptr::null_mut()).ok()?;
      let frame = frame?;
      frame.Initialize(None).ok()?;
      frame.SetSize(w as u32, h as u32).ok()?;
      // JPEG não tem alfa: manda BGR.
      let (mut format, data, stride): (GUID, Vec<u8>, u32) = if png {
        (GUID_WICPixelFormat32bppBGRA, bgra.to_vec(), (w * 4) as u32)
      } else {
        let bgr: Vec<u8> = bgra.as_chunks::<4>().0.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
        (GUID_WICPixelFormat24bppBGR, bgr, (w * 3) as u32)
      };
      frame.SetPixelFormat(&mut format).ok()?;
      frame.WritePixels(h as u32, stride, &data).ok()?;
      frame.Commit().ok()?;
      encoder.Commit().ok()?;
      let mut stat = Default::default();
      stream.Stat(&mut stat, STATFLAG_NONAME).ok()?;
      stream.Seek(0, STREAM_SEEK_SET, None).ok()?;
      let mut out = vec![0u8; stat.cbSize as usize];
      let mut read = 0u32;
      stream.Read(out.as_mut_ptr() as _, out.len() as u32, Some(&mut read)).ok().ok()?;
      out.truncate(read as usize);
      Some(out)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fit_mantem_a_proporcao() {
    assert_eq!(fit(1920, 1080, 320, 180), (320, 180));
    assert_eq!(fit(1000, 1000, 320, 180), (180, 180));
    assert_eq!(fit(3440, 1440, 320, 180), (320, 134));
  }

  #[test]
  fn fit_nao_amplia_nem_zera() {
    assert_eq!(fit(100, 50, 320, 180), (100, 50));
    assert_eq!(fit(5000, 1, 320, 180), (320, 1));
    assert_eq!(fit(0, 0, 320, 180), (320, 180));
  }

  #[test]
  fn preto() {
    assert!(is_black(&[0, 0, 0, 255].repeat(100)));
    let mut px = [0u8, 0, 0, 255].repeat(100);
    px[0] = 200;
    assert!(!is_black(&px));
  }
}
