import { useEffect, useState } from 'react';
import { useAuth } from '@clerk/react';
import { Workbench } from '@paulbreuler/shell';
import { bootstrapWorkbench } from '@/workbench/bootstrap';
import { useThemeStore } from '@/stores/useThemeStore';
import { useClerkSync } from '@/hooks/useClerkSync';
import { OAuthCallback } from '@/components/auth/OAuthCallback';
import { AuthErrorBoundary } from '@/components/auth/ErrorBoundary';

export function App() {
  const [ready, setReady] = useState(false);
  const { isSignedIn } = useAuth();

  useEffect(() => {
    useThemeStore.getState().setTheme(useThemeStore.getState().theme);
    bootstrapWorkbench().then(() => setReady(true));
  }, []);

  if (!ready) {
    return (
      <div className="flex items-center justify-center h-screen bg-bg-app text-text-muted">
        Loading…
      </div>
    );
  }

  return (
    <AuthErrorBoundary>
      <OAuthCallback />
      {isSignedIn && <ClerkTokenSync />}
      <Workbench />
    </AuthErrorBoundary>
  );
}

/** Syncs Clerk session token to Rust — only renders when signed in. */
function ClerkTokenSync() {
  useClerkSync();
  return null;
}
