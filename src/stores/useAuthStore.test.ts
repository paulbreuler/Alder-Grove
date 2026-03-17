import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useAuthStore, type AuthStatus } from './useAuthStore';

// Mock @tauri-apps/api — not available in test environment
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe('useAuthStore', () => {
  beforeEach(() => {
    useAuthStore.setState({
      status: 'idle',
      error: null,
    });
  });

  it('starts in idle state', () => {
    expect(useAuthStore.getState().status).toBe('idle');
    expect(useAuthStore.getState().error).toBeNull();
  });

  it('setStatus transitions state', () => {
    useAuthStore.getState().setStatus('authenticating');
    expect(useAuthStore.getState().status).toBe('authenticating');
  });

  it('setError sets error and status', () => {
    useAuthStore.getState().setError('OAuth failed');
    expect(useAuthStore.getState().status).toBe('error');
    expect(useAuthStore.getState().error).toBe('OAuth failed');
  });

  it('clearError resets to idle', () => {
    useAuthStore.getState().setError('something');
    useAuthStore.getState().clearError();
    expect(useAuthStore.getState().status).toBe('idle');
    expect(useAuthStore.getState().error).toBeNull();
  });
});
