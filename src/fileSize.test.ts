import { describe, expect, it } from 'vitest';
import { formatFileSize } from './fileSize';

describe('file sizes', () => {
  it('formats empty files, bytes, and binary units', () => {
    expect(formatFileSize(0)).toBe('0 B');
    expect(formatFileSize(1023)).toBe('1023 B');
    expect(formatFileSize(1024)).toBe('1 KiB');
    expect(formatFileSize(1536)).toBe('1.5 KiB');
    expect(formatFileSize(1024 ** 3)).toBe('1 GiB');
  });
});
