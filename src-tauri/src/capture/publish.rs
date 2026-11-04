// Fila controlada pelo host; não é um ACK da fila interna do renderer.
use std::{collections::VecDeque, time::{Duration, Instant}};
use serde_json::Value;

pub const MAX_AGE: Duration = Duration::from_millis(200);
pub const MAX_PCM_BYTES: usize = 48_000 * 2 * 4 / 5;

pub enum Message { Frame(Value), Pcm(Vec<u8>) }

#[derive(Default)]
pub struct PublishQueue {
  pub stopped: bool,
  pub scheduled: bool,
  ended: bool,
  generation: u64,
  last_seq: Option<u64>,
  frame: Option<(Instant, Value)>,
  pcm: VecDeque<(Instant, Vec<u8>)>,
  pcm_bytes: usize,
  controls: VecDeque<Value>,
  pub dropped_frames: u64,
  pub dropped_audio_bytes: u64,
}

impl PublishQueue {
  pub fn frame(&mut self, value: Value, now: Instant) {
    if self.stopped || self.ended { return; }
    if value["t"] != "frame" {
      if value["t"] == "ended" {
        self.ended = true;
        self.frame = None;
        self.pcm.clear();
        self.pcm_bytes = 0;
      }
      self.controls.push_back(value);
      return;
    }
    let (Some(generation), Some(seq), Some(slot)) = (value["gen"].as_u64(), value["seq"].as_u64(), value["slot"].as_u64()) else { return; };
    if generation < self.generation || slot > 2 || value["ts"].as_u64().is_none() || self.last_seq.is_some_and(|s| seq <= s) { return; }
    self.generation(generation);
    self.last_seq = Some(seq);
    if self.frame.replace((now, value)).is_some() { self.dropped_frames += 1; }
  }

  pub fn generation(&mut self, generation: u64) {
    if generation > self.generation {
      self.generation = generation;
      self.frame = None;
    }
  }

  pub fn pcm(&mut self, bytes: &[u8], now: Instant) {
    if self.stopped || self.ended || bytes.is_empty() || bytes.len() % 8 != 0 { return; }
    let kept = bytes.len().min(MAX_PCM_BYTES);
    self.dropped_audio_bytes += (bytes.len() - kept) as u64;
    self.pcm.push_back((now, bytes[bytes.len() - kept..].to_vec()));
    self.pcm_bytes += kept;
    while self.pcm_bytes > MAX_PCM_BYTES {
      if let Some((_, old)) = self.pcm.pop_front() {
        self.pcm_bytes -= old.len();
        self.dropped_audio_bytes += old.len() as u64;
      }
    }
  }

  pub fn pending(&self) -> bool { !self.stopped && (self.frame.is_some() || !self.pcm.is_empty() || !self.controls.is_empty()) }

  pub fn take(&mut self, now: Instant) -> Vec<Message> {
    if self.stopped { return vec![]; }
    let mut out: Vec<Message> = self.controls.drain(..).map(Message::Frame).collect();
    if let Some((time, value)) = self.frame.take() {
      if now.saturating_duration_since(time) <= MAX_AGE { out.push(Message::Frame(value)); }
      else { self.dropped_frames += 1; }
    }
    while let Some((time, bytes)) = self.pcm.pop_front() {
      if now.saturating_duration_since(time) <= MAX_AGE { out.push(Message::Pcm(bytes)); }
      else { self.dropped_audio_bytes += bytes.len() as u64; }
    }
    self.pcm_bytes = 0;
    out
  }

  pub fn stop(&mut self) {
    self.stopped = true;
    self.frame = None;
    self.pcm.clear();
    self.controls.clear();
    self.pcm_bytes = 0;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  fn frame(seq: u64, generation: u64) -> Value { json!({"t":"frame","gen":generation,"seq":seq,"slot":seq%3,"ts":seq*10000}) }

  #[test]
  fn so_publica_o_frame_mais_recente_e_preserva_controle() {
    let now = Instant::now(); let mut q = PublishQueue::default();
    q.frame(json!({"t":"warn","reason":"black"}), now);
    for seq in 0..100 { q.frame(frame(seq, 1), now); }
    let messages = q.take(now);
    assert_eq!(messages.len(), 2);
    assert!(matches!(&messages[1], Message::Frame(v) if v["seq"] == 99));
    assert_eq!(q.dropped_frames, 99);
  }
  #[test]
  fn limita_pcm_a_200ms_e_descarta_idade() {
    let now = Instant::now(); let mut q = PublishQueue::default();
    for _ in 0..100 { q.pcm(&vec![0; 480 * 2 * 4], now); }
    assert_eq!(q.pcm_bytes, MAX_PCM_BYTES);
    assert_eq!(q.take(now + Duration::from_secs(1)).len(), 0);
    assert_eq!(q.pcm_bytes, 0);
  }
  #[test]
  fn ended_e_stop_sao_terminais() {
    let now = Instant::now(); let mut q = PublishQueue::default();
    q.frame(frame(0, 1), now); q.pcm(&[0; 8], now);
    q.frame(json!({"t":"ended","reason":"stopped"}), now);
    q.frame(frame(1, 1), now); q.pcm(&[0; 8], now);
    assert_eq!(q.take(now).len(), 1);
    q.stop(); q.frame(frame(2, 1), now);
    assert!(!q.pending()); assert!(q.take(now).is_empty());
  }
  #[test]
  fn invalida_geracao_antiga_e_frame_velho() {
    let now = Instant::now(); let mut q = PublishQueue::default();
    q.frame(frame(0, 1), now); q.generation(2);
    q.frame(frame(1, 1), now); assert!(q.take(now).is_empty());
    q.frame(frame(2, 2), now);
    assert!(q.take(now + Duration::from_secs(1)).is_empty());
  }
}
