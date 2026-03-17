import { useEffect, useState } from 'react';
import { Show, SignInButton } from '@clerk/react';
import { Workbench } from '@paulbreuler/shell';
import { bootstrapWorkbench } from '@/workbench/bootstrap';
import { useThemeStore } from '@/stores/useThemeStore';

export function App() {
  return (
    <Show when="signed-in" fallback={<SignInPage />}>
      <AuthenticatedApp />
    </Show>
  );
}

function AuthenticatedApp() {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    useThemeStore.getState().setTheme(useThemeStore.getState().theme);
    bootstrapWorkbench().then(() => setReady(true));
  }, []);

  if (!ready) {
    return (
      <div className="flex items-center justify-center h-screen bg-[var(--grove-surface-base)] text-[var(--grove-text-secondary)]">
        Loading…
      </div>
    );
  }

  return <Workbench />;
}

function SignInPage() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-[var(--grove-space-6)] bg-[var(--grove-surface-base)] text-[var(--grove-text-primary)] font-[var(--grove-font-sans)]">
      <h1 className="m-0">Alder Grove</h1>
      <p className="m-0 text-[var(--grove-text-secondary)]">
        Your applications grow in the Grove.
      </p>
      <SignInButton mode="modal">
        <button className="py-[var(--grove-space-2)] px-[var(--grove-space-6)] bg-[var(--grove-accent)] text-[var(--grove-text-primary)] border-none rounded-[var(--grove-radius-md)] cursor-pointer font-[var(--grove-font-sans)] text-[var(--grove-font-size-base)]">
          Sign in
        </button>
      </SignInButton>
    </div>
  );
}
