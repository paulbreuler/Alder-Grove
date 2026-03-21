import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the shell's globalExtensionRegistry
const mockActivate = vi.fn().mockResolvedValue(undefined);
vi.mock('@paulbreuler/shell', () => ({
  globalExtensionRegistry: {
    activate: mockActivate,
  },
}));

describe('bootstrapShell', () => {
  beforeEach(() => {
    mockActivate.mockClear();
    vi.resetModules();
  });

  it('activates the grove core extension', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell();

    expect(mockActivate).toHaveBeenCalledTimes(1);
    const extension = mockActivate.mock.calls[0][0];
    expect(extension.id).toBe('grove.core');
  });

  it('is idempotent — second call does not activate again', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell();
    await bootstrapShell();

    expect(mockActivate).toHaveBeenCalledTimes(1);
  });

  it('always activates core even when isEnabled rejects it', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell(() => false);

    // Core is always activated regardless of isEnabled
    expect(mockActivate).toHaveBeenCalledTimes(1);
    expect(mockActivate.mock.calls[0][0].id).toBe('grove.core');
  });

  it('activates all extensions when isEnabled returns true', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell(() => true);

    expect(mockActivate).toHaveBeenCalledTimes(1);
    expect(mockActivate.mock.calls[0][0].id).toBe('grove.core');
  });
});
