import { useEffect, useState } from 'react';
import { useAuth } from '@clerk/react';
import { Workbench, TitleBar } from '@paulbreuler/shell';
import { bootstrapWorkbench } from '@/workbench/bootstrap';
import { useThemeStore } from '@/stores/useThemeStore';
import { useClerkSync } from '@/hooks/useClerkSync';
import { LoginScreen } from '@/components/auth/LoginScreen';
import { OAuthCallback } from '@/components/auth/OAuthCallback';
import { AuthErrorBoundary } from '@/components/auth/ErrorBoundary';

export function App() {
  const { isSignedIn, isLoaded } = useAuth();

  return (
    <div
      className="flex flex-col h-screen"
      style={{ backgroundColor: 'var(--grove-surface-base)' }}
    >
      <TitleBar title="Alder Grove" />
      <OAuthCallback />
      <div className="flex-1 min-h-0">
        <AuthErrorBoundary>
          {!isLoaded ? (
            <div
              className="flex items-center justify-center h-full"
              style={{ color: 'var(--grove-text-muted)' }}
            >
              Loading...
            </div>
          ) : isSignedIn ? (
            <AuthenticatedApp />
          ) : (
            <LoginScreen />
          )}
        </AuthErrorBoundary>
      </div>
    </div>
  );
}

function AuthenticatedApp() {
  const [ready, setReady] = useState(false);
  useClerkSync();

  useEffect(() => {
    useThemeStore.getState().setTheme(useThemeStore.getState().theme);
    bootstrapWorkbench().then(() => setReady(true));
  }, []);

  if (!ready) {
    return (
      <div
        className="flex items-center justify-center h-full"
        style={{ color: 'var(--grove-text-secondary)' }}
      >
        Loading...
      </div>
    );
  }

  return <Workbench />;
}
