import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { App } from './App';

let mockIsSignedIn = false;

vi.mock('@clerk/react', () => ({
  useAuth: () => ({ isSignedIn: mockIsSignedIn, isLoaded: true }),
  useClerk: () => ({ client: { signIn: { create: vi.fn() } }, session: null, handleRedirectCallback: vi.fn() }),
  useUser: () => ({ user: null }),
  useSession: () => ({ session: null }),
  UserButton: () => <div data-testid="user-button" />,
  ClerkProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock('@paulbreuler/shell', () => ({
  Workbench: () => <div data-testid="workbench">Workbench</div>,
  globalExtensionRegistry: { activate: vi.fn().mockResolvedValue(undefined) },
}));

vi.mock('@/workbench/bootstrap', () => ({
  bootstrapWorkbench: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));

function renderApp() {
  return render(<MemoryRouter><App /></MemoryRouter>);
}

describe('App', () => {
  beforeEach(() => {
    mockIsSignedIn = false;
  });

  it('renders Workbench after bootstrap', async () => {
    renderApp();
    await waitFor(() => {
      expect(screen.getByTestId('workbench')).toBeInTheDocument();
    });
  });

  it('renders Workbench regardless of auth state', async () => {
    mockIsSignedIn = true;
    renderApp();
    await waitFor(() => {
      expect(screen.getByTestId('workbench')).toBeInTheDocument();
    });
  });
});
