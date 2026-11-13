//! Vídeo de teste gerado em código: barras de cor em movimento com contador de quadros.
//! Palco 1920x1080 e duas miniaturas 400x225, 60 quadros por segundo, entregues à tela como
//! imagem RGBA por quadro (R3: `SharedPixelBuffer` -> `Image::from_rgba8_premultiplied`).
//! Cores só dos tokens (tokens.slint / tokens.css).

use crate::{Contadores, RoomWindow};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub const FPS: u32 = 60;

fn cor(hex: u32) -> Rgba8Pixel {
    Rgba8Pixel::new((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255)
}

/// Fonte de 5x7 para os dígitos 0-9 (cada linha: 5 bits).
const DIGITOS: [[u8; 7]; 10] = [
    [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
    [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
    [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
    [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
    [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
    [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
    [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
];

/// Gerador de um quadro: faixa de barras que rola 8 px (por 1920) a cada quadro.
struct Gerador {
    w: usize,
    h: usize,
    /// Duas voltas das barras, para copiar uma linha com qualquer deslocamento sem costura.
    linha: Vec<Rgba8Pixel>,
    periodo: usize,
    passo: usize,
    caixa: Rgba8Pixel,
    digito: Rgba8Pixel,
    escala: usize,
}

impl Gerador {
    fn novo(w: usize, h: usize, barras: &[u32], escala: usize, passo: usize) -> Self {
        let periodo = w;
        let larg = periodo / barras.len();
        let mut linha = Vec::with_capacity(periodo * 2);
        for x in 0..periodo * 2 {
            linha.push(cor(barras[(x % periodo) / larg % barras.len()]));
        }
        Gerador { w, h, linha, periodo, passo, caixa: cor(0x12110f), digito: cor(0xf5ead8), escala }
    }

    fn quadro(&self, n: u64) -> SharedPixelBuffer<Rgba8Pixel> {
        let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(self.w as u32, self.h as u32);
        let px = buf.make_mut_slice();
        let desloc = (n as usize * self.passo) % self.periodo;
        let fatia = &self.linha[desloc..desloc + self.w];
        for linha in px.chunks_exact_mut(self.w) {
            linha.copy_from_slice(fatia);
        }
        self.contador(px, n);
        buf
    }

    /// Caixa escura no centro com o número do quadro (8 dígitos).
    fn contador(&self, px: &mut [Rgba8Pixel], n: u64) {
        let e = self.escala;
        let cel = 6 * e; // 5 colunas + 1 de espaço
        let larg = 8 * cel + 2 * e;
        let alt = 7 * e + 2 * e * 2;
        let x0 = (self.w - larg) / 2;
        let y0 = (self.h - alt) / 2;
        for y in y0..y0 + alt {
            px[y * self.w + x0..y * self.w + x0 + larg].fill(self.caixa);
        }
        let mut resto = n % 100_000_000;
        let mut ds = [0usize; 8];
        for d in ds.iter_mut().rev() {
            *d = (resto % 10) as usize;
            resto /= 10;
        }
        for (i, d) in ds.iter().enumerate() {
            for (ly, bits) in DIGITOS[*d].iter().enumerate() {
                for lx in 0..5 {
                    if bits & (1 << (4 - lx)) != 0 {
                        let x = x0 + e + i * cel + lx * e;
                        let y = y0 + 2 * e + ly * e;
                        for yy in y..y + e {
                            px[yy * self.w + x..yy * self.w + x + e].fill(self.digito);
                        }
                    }
                }
            }
        }
    }
}

/// Liga o thread que gera os quadros e os entrega à janela, no ritmo de 60 por segundo.
pub fn iniciar(ui: Weak<RoomWindow>, c: Arc<Contadores>, parar: Arc<AtomicBool>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        // Barras: marrom e verde do tema (accent-*, accent-2-*, neutral-300).
        let palco = Gerador::novo(
            1920,
            1080,
            &[0xc67139, 0x7a8a5e, 0xf6a06b, 0xaebf92, 0xffc6a5, 0x8c491a, 0xdcd3c4, 0x3d472b],
            14,
            8,
        );
        let mini_a = Gerador::novo(400, 225, &[0x643312, 0x8c491a, 0x402310, 0xb2622d], 5, 3);
        let mini_b = Gerador::novo(400, 225, &[0x3d472b, 0x7a8a5e, 0x272e1b, 0xaebf92], 5, 3);

        // A tela ainda não consumiu o quadro anterior? Joga fora este (não enfileira).
        let ocupada = Arc::new(AtomicBool::new(false));
        let periodo = Duration::from_secs_f64(1.0 / FPS as f64);
        let inicio = Instant::now();
        let mut n: u64 = 0;
        while !parar.load(Ordering::Relaxed) {
            let alvo = inicio + periodo * n as u32;
            let agora = Instant::now();
            if alvo > agora {
                std::thread::sleep(alvo - agora);
            }
            if ocupada.load(Ordering::Acquire) {
                c.descartados.fetch_add(1, Ordering::Relaxed);
            } else {
                let (b0, b1, b2) = (palco.quadro(n), mini_a.quadro(n), mini_b.quadro(n + 1000));
                ocupada.store(true, Ordering::Release);
                let (ocupada, c, ui) = (ocupada.clone(), c.clone(), ui.clone());
                let enviado = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui.upgrade() {
                        ui.set_stage_frame(Image::from_rgba8_premultiplied(b0));
                        ui.set_thumb_a(Image::from_rgba8_premultiplied(b1));
                        ui.set_thumb_b(Image::from_rgba8_premultiplied(b2));
                        c.entregues.fetch_add(1, Ordering::Relaxed);
                    }
                    ocupada.store(false, Ordering::Release);
                });
                if enviado.is_err() {
                    break;
                }
            }
            n += 1;
        }
    })
}
