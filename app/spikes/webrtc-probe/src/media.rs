//! Fontes de vídeo e som GERADAS (padrão de teste e tom) e os consumidores do que chega do outro lado.
//! Nada aqui captura tela, câmera ou microfone.
//!
//! Pulso de sincronia: uma vez por segundo a imagem tem um clarão (faixa inferior branca por 100 ms) e o som
//! tem um bip de 1 kHz (primeiros 100 ms do segundo). Com duas sondas (uma envia, outra recebe), a que recebe
//! mede a distância entre clarão e bip nos quadros que chegam (`--detect-pulse`).

use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use libwebrtc::audio_frame::AudioFrame;
use libwebrtc::audio_source::native::NativeAudioSource;
use libwebrtc::audio_source::AudioSourceOptions;
use libwebrtc::audio_stream::native::NativeAudioStream;
use libwebrtc::audio_track::RtcAudioTrack;
use libwebrtc::video_frame::{I420Buffer, VideoBuffer, VideoFrame, VideoRotation};
use libwebrtc::video_source::native::NativeVideoSource;
use libwebrtc::video_stream::native::NativeVideoStream;
use libwebrtc::video_track::RtcVideoTrack;

const AUDIO_RATE: u32 = 48_000;
const FLASH_MS: u64 = 100;

// ---------------------------------------------------------------- geração de vídeo

struct Template {
    w: usize,
    h: usize,
    y: Vec<u8>,
    u: Vec<u8>,
    v: Vec<u8>,
    sy: usize,
    su: usize,
    sv: usize,
}

/// Barras de cor (75% de cima), degradê (17%) e faixa do clarão, preta (8% de baixo).
fn build_template(w: u32, h: u32) -> Template {
    let base = I420Buffer::new(w, h);
    let (sy, su, sv) = base.strides();
    let (sy, su, sv) = (sy as usize, su as usize, sv as usize);
    let (w, h) = (w as usize, h as usize);
    let (cw, ch) = (w / 2, h / 2);
    let mut y = vec![16u8; sy * h];
    let mut u = vec![128u8; su * ch];
    let mut v = vec![128u8; sv * ch];
    // Barras SMPTE simplificadas (Y, U, V em BT.601 de faixa limitada).
    const BARS: [(u8, u8, u8); 7] = [
        (180, 128, 128), // cinza
        (162, 44, 142),  // amarelo
        (131, 156, 44),  // ciano
        (112, 72, 58),   // verde
        (84, 184, 198),  // magenta
        (65, 100, 212),  // vermelho
        (35, 212, 114),  // azul
    ];
    let bars_h = h * 75 / 100;
    for row in 0..bars_h {
        for col in 0..w {
            let b = BARS[col * 7 / w];
            y[row * sy + col] = b.0;
        }
    }
    for row in 0..bars_h / 2 {
        for col in 0..cw {
            let b = BARS[col * 7 / cw];
            u[row * su + col] = b.1;
            v[row * sv + col] = b.2;
        }
    }
    let grad_end = h * 92 / 100;
    for row in bars_h..grad_end {
        for col in 0..w {
            y[row * sy + col] = (16 + col * 219 / w) as u8;
        }
    }
    Template { w, h, y, u, v, sy, su, sv }
}

/// Gera quadros em uma thread própria, no ritmo `fps`, até `stop`.
pub fn spawn_video_generator(source: NativeVideoSource, w: u32, h: u32, fps: u32, stop: Arc<AtomicBool>, frames_out: Arc<AtomicU64>) {
    std::thread::Builder::new()
        .name("gerador-de-video".into())
        .spawn(move || {
            let t = build_template(w, h);
            let period = Duration::from_secs_f64(1.0 / fps as f64);
            let t0 = Instant::now();
            let mut next = t0;
            let mut n: u64 = 0;
            let flash_row0 = t.h * 93 / 100;
            let flash_row1 = t.h * 99 / 100;
            while !stop.load(Ordering::Relaxed) {
                let elapsed = t0.elapsed();
                let mut buf = I420Buffer::new(w, h);
                {
                    let (dy, du, dv) = buf.data_mut();
                    let l = dy.len().min(t.y.len());
                    dy[..l].copy_from_slice(&t.y[..l]);
                    let l = du.len().min(t.u.len());
                    du[..l].copy_from_slice(&t.u[..l]);
                    let l = dv.len().min(t.v.len());
                    dv[..l].copy_from_slice(&t.v[..l]);

                    // Contador binário de 16 bits (quadrados de 40 px) no canto superior esquerdo.
                    let sq = 40usize.min(t.w / 40).max(4);
                    for bit in 0..16usize {
                        let on = (n >> (15 - bit)) & 1 == 1;
                        let val = if on { 235 } else { 16 };
                        let x0 = 20 + bit * (sq + 4);
                        for row in 20..(20 + sq).min(t.h) {
                            for col in x0..(x0 + sq).min(t.w) {
                                dy[row * t.sy + col] = val;
                            }
                        }
                    }
                    // Barra branca andando (16 px de largura), nas barras de cor.
                    let bar_x = ((n * 12) % (t.w as u64 - 16)) as usize;
                    for row in 0..t.h * 75 / 100 {
                        for col in bar_x..bar_x + 16 {
                            dy[row * t.sy + col] = 235;
                        }
                    }
                    // Clarão de 1 s: faixa inferior branca nos primeiros 100 ms do segundo.
                    if (elapsed.as_millis() as u64) % 1000 < FLASH_MS {
                        for row in flash_row0..flash_row1 {
                            for col in 0..t.w {
                                dy[row * t.sy + col] = 235;
                            }
                        }
                    }
                    let _ = (t.su, t.sv);
                }
                let mut frame = VideoFrame::new(VideoRotation::VideoRotation0, buf);
                frame.timestamp_us = elapsed.as_micros() as i64;
                source.capture_frame(&frame);
                n += 1;
                frames_out.store(n, Ordering::Relaxed);

                next += period;
                let now = Instant::now();
                if next > now {
                    std::thread::sleep(next - now);
                } else if now - next > period * 5 {
                    // Ficou muito atrasado (máquina ocupada): não tenta recuperar todos os quadros.
                    next = now;
                }
            }
        })
        .expect("thread do gerador de vídeo");
}

// ---------------------------------------------------------------- geração de som

/// Tom de 440 Hz (quieto) e bip de 1 kHz (alto) nos primeiros 100 ms de cada segundo, em mono 48 kHz.
pub fn spawn_audio_generator(source: NativeAudioSource, stop: Arc<AtomicBool>, samples_out: Arc<AtomicU64>) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(10));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
        let mut n: u64 = 0; // amostras já enviadas
        const PER_FRAME: usize = (AUDIO_RATE / 100) as usize;
        while !stop.load(Ordering::Relaxed) {
            tick.tick().await;
            let mut data = Vec::with_capacity(PER_FRAME);
            for i in 0..PER_FRAME as u64 {
                let k = n + i;
                let t = k as f64 / AUDIO_RATE as f64;
                let phase_ms = (k % AUDIO_RATE as u64) * 1000 / AUDIO_RATE as u64;
                let (hz, amp) = if phase_ms < FLASH_MS { (1000.0, 0.5) } else { (440.0, 0.1) };
                let s = (2.0 * std::f64::consts::PI * hz * t).sin() * amp * i16::MAX as f64;
                data.push(s as i16);
            }
            let frame = AudioFrame {
                data: Cow::Owned(data),
                sample_rate: AUDIO_RATE,
                num_channels: 1,
                samples_per_channel: PER_FRAME as u32,
            };
            if source.capture_frame(&frame).await.is_err() {
                break;
            }
            n += PER_FRAME as u64;
            samples_out.store(n, Ordering::Relaxed);
        }
    });
}

pub fn audio_source() -> NativeAudioSource {
    // Sem cancelamento de eco, ruído ou ganho: é um tom, não voz. Fila de 100 ms.
    NativeAudioSource::new(AudioSourceOptions { echo_cancellation: false, noise_suppression: false, auto_gain_control: false }, AUDIO_RATE, 1, 100)
}

// ---------------------------------------------------------------- recepção

#[derive(Default)]
pub struct PulseState {
    flash_on: bool,
    beep_on: bool,
    last_flash: Option<Instant>,
    last_beep: Option<Instant>,
    flash_paired: bool,
    beep_paired: bool,
    /// som menos imagem, em ms (positivo = som depois da imagem).
    pub skews_ms: Vec<f64>,
}

#[derive(Default)]
pub struct RxCounters {
    pub video_frames: AtomicU64,
    pub video_w: AtomicU32,
    pub video_h: AtomicU32,
    pub video_non_i420: AtomicU64,
    pub audio_frames: AtomicU64,
    pub audio_samples: AtomicU64,
    pub pulse: Mutex<PulseState>,
}

impl PulseState {
    fn on_flash(&mut self, now: Instant) {
        self.last_flash = Some(now);
        self.flash_paired = false;
        self.try_pair();
    }
    fn on_beep(&mut self, now: Instant) {
        self.last_beep = Some(now);
        self.beep_paired = false;
        self.try_pair();
    }
    fn try_pair(&mut self) {
        if let (Some(f), Some(b)) = (self.last_flash, self.last_beep) {
            if self.flash_paired || self.beep_paired {
                return;
            }
            let d = if b >= f { b.duration_since(f).as_secs_f64() } else { -(f.duration_since(b).as_secs_f64()) } * 1000.0;
            if d.abs() < 500.0 {
                self.skews_ms.push(d);
                self.flash_paired = true;
                self.beep_paired = true;
            }
        }
    }
}

/// Média da luminância da faixa do clarão (amostrando 1 pixel a cada 8).
fn flash_level(i420: &I420Buffer) -> f64 {
    let (w, h) = (i420.width() as usize, i420.height() as usize);
    let (sy, _, _) = i420.strides();
    let sy = sy as usize;
    let (y, _, _) = i420.data();
    let (r0, r1) = (h * 93 / 100, h * 99 / 100);
    let (mut sum, mut cnt) = (0u64, 0u64);
    for row in (r0..r1).step_by(4) {
        for col in (0..w).step_by(8) {
            sum += y[row * sy + col] as u64;
            cnt += 1;
        }
    }
    if cnt == 0 {
        0.0
    } else {
        sum as f64 / cnt as f64
    }
}

pub fn spawn_video_sink(track: RtcVideoTrack, c: Arc<RxCounters>, detect: bool) {
    tokio::spawn(async move {
        let mut stream = NativeVideoStream::new(track);
        while let Some(frame) = stream.next().await {
            c.video_frames.fetch_add(1, Ordering::Relaxed);
            c.video_w.store(frame.buffer.width(), Ordering::Relaxed);
            c.video_h.store(frame.buffer.height(), Ordering::Relaxed);
            if detect {
                // `to_i420` é privado da crate: só dá para olhar os pixels de quadros que já chegam em I420.
                match frame.buffer.as_i420() {
                    Some(i) => {
                        let lvl = flash_level(i);
                        let mut p = c.pulse.lock().unwrap();
                        if !p.flash_on && lvl > 180.0 {
                            p.flash_on = true;
                            p.on_flash(Instant::now());
                        } else if p.flash_on && lvl < 100.0 {
                            p.flash_on = false;
                        }
                    }
                    None => {
                        c.video_non_i420.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    });
}

pub fn spawn_audio_sink(track: RtcAudioTrack, c: Arc<RxCounters>, detect: bool) {
    tokio::spawn(async move {
        let mut stream = NativeAudioStream::new(track, AUDIO_RATE as i32, 1);
        while let Some(frame) = stream.next().await {
            c.audio_frames.fetch_add(1, Ordering::Relaxed);
            c.audio_samples.fetch_add(frame.samples_per_channel as u64, Ordering::Relaxed);
            if detect {
                let peak = frame.data.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0);
                let mut p = c.pulse.lock().unwrap();
                if !p.beep_on && peak > 12_000 {
                    p.beep_on = true;
                    p.on_beep(Instant::now());
                } else if p.beep_on && peak < 8_000 {
                    p.beep_on = false;
                }
            }
        }
    });
}
