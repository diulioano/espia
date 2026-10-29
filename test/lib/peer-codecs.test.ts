import { describe, expect, it } from 'vitest';
import { orderCodecs } from '../../src/lib/peer';

const c = (mimeType: string, sdpFmtpLine?: string) => ({ mimeType, sdpFmtpLine });

describe('orderCodecs', () => {
  it('põe H.264 (packetization-mode=1 antes) na frente e mantém o resto na ordem', () => {
    const vp8 = c('video/VP8');
    const rtx = c('video/rtx', 'apt=96');
    const h264m0 = c('video/H264', 'level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=42e01f');
    const h264m1 = c('video/H264', 'level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f');
    const vp9 = c('video/VP9', 'profile-id=0');
    expect(orderCodecs([vp8, rtx, h264m0, h264m1, vp9])).toEqual([h264m1, h264m0, vp8, rtx, vp9]);
  });

  it('sem H.264 não muda nada', () => {
    const list = [c('video/VP8'), c('video/VP9'), c('video/AV1')];
    expect(orderCodecs(list)).toEqual(list);
  });
});
