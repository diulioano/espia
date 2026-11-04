// Tempo da mídia dentro de uma captura, sempre no domínio monotônico da página (µs).
export const MAX_MEDIA_AGE_US = 200_000;

export class CaptureTiming {
  private videoOffsetUs: number | null = null;
  private lastVideoUs = -1;
  private lastNativeTsUs = -1;
  private lastSeq = -1;
  private audioNextUs: number | null = null;
  private lastAudioUs = -1;
  private stopped = false;

  constructor(private readonly nowUs = () => Math.round(performance.now() * 1000)) {}

  video(ts: number, seq: number): number | null {
    if (this.stopped || !Number.isSafeInteger(ts) || ts < 0 || !Number.isSafeInteger(seq) || seq < 0 ||
        ts <= this.lastNativeTsUs || seq <= this.lastSeq) return null;
    const now = this.nowUs();
    this.videoOffsetUs ??= now - ts;
    const timestamp = this.videoOffsetUs + ts;
    if (now - timestamp > MAX_MEDIA_AGE_US || timestamp <= this.lastVideoUs) return null;
    this.lastNativeTsUs = ts;
    this.lastSeq = seq;
    this.lastVideoUs = timestamp;
    return timestamp;
  }

  audio(frames: number): number | null {
    if (this.stopped || !Number.isSafeInteger(frames) || frames <= 0 || frames > 9600) return null;
    const now = this.nowUs();
    // Reancora apenas depois de uma lacuna: não remarca todo pacote atrasado como novo.
    if (this.audioNextUs === null || now - this.audioNextUs > MAX_MEDIA_AGE_US) {
      this.audioNextUs = Math.max(now, this.lastAudioUs + 1);
    }
    // Burst/FIFO não pode agendar segundos de som no futuro.
    if (this.audioNextUs - now >= MAX_MEDIA_AGE_US) return null;
    const timestamp = this.audioNextUs;
    this.audioNextUs += Math.round(frames * 1_000_000 / 48_000);
    this.lastAudioUs = timestamp;
    return timestamp;
  }

  resetAudio() { this.audioNextUs = null; }
  stop() { this.stopped = true; }
}
