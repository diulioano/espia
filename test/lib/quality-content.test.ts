import { expect, it } from 'vitest';
import { contentPreference } from '../../src/lib/quality';
it('uses content hints before source fallback without changing chosen resolution or fps', () => {
    expect(contentPreference('motion', 'screen')).toBe('maintain-framerate');
    expect(contentPreference('text', 'camera')).toBe('maintain-resolution');
    expect(contentPreference('', 'camera')).toBe('maintain-framerate');
    expect(contentPreference('', 'screen')).toBe('maintain-resolution');
});
