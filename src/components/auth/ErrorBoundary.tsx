import { Component, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: (error: Error, reset: () => void) => ReactNode;
}

interface State {
  error: Error | null;
}

/**
 * Catches React render errors (including Clerk component failures)
 * and shows a recovery UI instead of a white screen.
 */
export class AuthErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  reset = (): void => {
    this.setState({ error: null });
  };

  render(): ReactNode {
    if (this.state.error) {
      if (this.props.fallback) {
        return this.props.fallback(this.state.error, this.reset);
      }
      return (
        <div
          className="flex flex-col items-center justify-center h-full gap-[var(--grove-space-4)] p-[var(--grove-space-8)]"
          style={{ color: 'var(--grove-text-primary)' }}
        >
          <h2 className="text-[var(--grove-font-size-lg)] font-semibold">
            Something went wrong
          </h2>
          <p
            className="text-[var(--grove-font-size-sm)]"
            style={{ color: 'var(--grove-text-muted)' }}
          >
            {this.state.error.message}
          </p>
          <button
            onClick={this.reset}
            className="px-[var(--grove-space-4)] py-[var(--grove-space-2)] text-[var(--grove-font-size-sm)] rounded-[var(--grove-radius-md)]"
            style={{
              backgroundColor: 'var(--grove-surface-sunken)',
            }}
          >
            Try again
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
