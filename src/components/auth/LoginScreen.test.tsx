import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LoginScreen } from './LoginScreen';
import { useAuthStore } from '@/stores/useAuthStore';

vi.mock('@clerk/react', () => ({
  useSignIn: () => ({
    signIn: {
      sso: vi.fn().mockResolvedValue({ error: null }),
    },
    fetchStatus: 'idle',
  }),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('LoginScreen', () => {
  beforeEach(() => {
    useAuthStore.setState({ status: 'idle', error: null });
  });

  it('renders the title and sign-in options', () => {
    render(<LoginScreen />);
    expect(screen.getByText('Alder Grove')).toBeInTheDocument();
    expect(screen.getAllByText(/sign in/i).length).toBeGreaterThan(0);
  });

  it('renders OAuth provider buttons', () => {
    render(<LoginScreen />);
    expect(screen.getByRole('button', { name: /apple/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /github/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /google/i })).toBeInTheDocument();
  });

  it('shows error when auth store has error', () => {
    useAuthStore.setState({ status: 'error', error: 'OAuth failed' });
    render(<LoginScreen />);
    expect(screen.getByText('OAuth failed')).toBeInTheDocument();
  });
});
