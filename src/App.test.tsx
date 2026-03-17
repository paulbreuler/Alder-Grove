import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { App } from './App';

let mockIsSignedIn = false;

vi.mock('@clerk/react', () => ({
  Show: ({
    when,
    fallback,
    children,
  }: {
    when: string;
    fallback?: React.ReactNode;
    children?: React.ReactNode;
  }) => {
    if (when === 'signed-in' && mockIsSignedIn) return <>{children}</>;
    if (when === 'signed-in' && !mockIsSignedIn) return <>{fallback}</>;
    return null;
  },
  SignInButton: ({ children }: { children?: React.ReactNode }) =>
    children ?? <button>Sign in</button>,
  ClerkProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock('@paulbreuler/shell', () => ({
  Workbench: () => <div data-testid="workbench">Workbench</div>,
}));

function renderApp() {
  return render(
    <MemoryRouter>
      <App />
    </MemoryRouter>,
  );
}

describe('App', () => {
  beforeEach(() => {
    mockIsSignedIn = false;
  });

  it('renders sign-in page when signed out', () => {
    renderApp();
    expect(screen.getByText('Alder Grove')).toBeInTheDocument();
    expect(screen.getByText('Sign in')).toBeInTheDocument();
  });

  it('renders the tagline when signed out', () => {
    renderApp();
    expect(
      screen.getByText('Your applications grow in the Grove.'),
    ).toBeInTheDocument();
  });

  it('renders Shell Workbench when signed in', () => {
    mockIsSignedIn = true;
    renderApp();
    expect(screen.getByTestId('workbench')).toBeInTheDocument();
  });

  it('does not render sign-in button when signed in', () => {
    mockIsSignedIn = true;
    renderApp();
    expect(screen.queryByText('Sign in')).not.toBeInTheDocument();
  });

  it('does not render workbench when signed out', () => {
    renderApp();
    expect(screen.queryByTestId('workbench')).not.toBeInTheDocument();
  });
});
