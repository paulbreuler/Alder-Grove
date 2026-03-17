import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { App } from './App';

let mockIsSignedIn = false;
let mockIsLoaded = true;

vi.mock('@clerk/react', () => ({
  useAuth: () => ({ isSignedIn: mockIsSignedIn, isLoaded: mockIsLoaded }),
  useClerk: () => ({ client: { signIn: { create: vi.fn() } }, session: null }),
  useSession: () => ({ session: null }),
  ClerkProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock('@paulbreuler/shell', () => ({
  Workbench: () => <div data-testid="workbench">Workbench</div>,
  TitleBar: ({ title }: { title: string }) => <div data-testid="title-bar">{title}</div>,
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
    mockIsLoaded = true;
  });

  it('always renders the title bar', () => {
    renderApp();
    expect(screen.getByTestId('title-bar')).toBeInTheDocument();
  });

  it('renders login screen when signed out', () => {
    renderApp();
    expect(screen.getByText('Your applications grow in the Grove.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /github/i })).toBeInTheDocument();
  });

  it('renders Workbench when signed in', async () => {
    mockIsSignedIn = true;
    renderApp();
    await waitFor(() => {
      expect(screen.getByTestId('workbench')).toBeInTheDocument();
    });
  });

  it('shows loading when Clerk not loaded', () => {
    mockIsLoaded = false;
    renderApp();
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });
});
