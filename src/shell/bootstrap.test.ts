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

  it('activates core + feature extensions', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell();

    // core (1) + 3 feature extensions (home, user, settings)
    expect(mockActivate).toHaveBeenCalledTimes(4);
    expect(mockActivate.mock.calls[0][0].id).toBe('grove.core');
    expect(mockActivate.mock.calls[1][0].id).toBe('grove.home');
    expect(mockActivate.mock.calls[2][0].id).toBe('grove.user');
    expect(mockActivate.mock.calls[3][0].id).toBe('grove.settings');
  });

  it('is idempotent — second call does not activate again', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell();
    await bootstrapShell();

    expect(mockActivate).toHaveBeenCalledTimes(4);
  });

  it('always activates core even when isEnabled rejects all features', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell(() => false);

    // Core is always activated regardless of isEnabled
    expect(mockActivate).toHaveBeenCalledTimes(1);
    expect(mockActivate.mock.calls[0][0].id).toBe('grove.core');
  });

  it('filters feature extensions via isEnabled', async () => {
    const { bootstrapShell } = await import('./bootstrap');
    await bootstrapShell((id) => id === 'grove.home');

    // core (always) + home (enabled) = 2
    expect(mockActivate).toHaveBeenCalledTimes(2);
    expect(mockActivate.mock.calls[0][0].id).toBe('grove.core');
    expect(mockActivate.mock.calls[1][0].id).toBe('grove.home');
  });
});
