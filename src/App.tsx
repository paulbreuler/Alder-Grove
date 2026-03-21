import { Show, SignInButton } from '@clerk/react';
import { Workbench } from '@paulbreuler/shell';
import { bootstrapShell } from './shell/bootstrap';

// Initialize shell extensions and theme before first render.
// This is idempotent — safe under React StrictMode and HMR.
void bootstrapShell().catch((error: unknown) => {
  console.error('[Grove] Shell bootstrap failed:', error);
});

/**
 * Root application component.
 *
 * When signed out, renders a branded sign-in page.
 * When signed in, renders the Alder Shell Workbench.
 */
export function App() {
  return (
    <Show
      when="signed-in"
      fallback={<SignInPage />}
    >
      <Workbench />
    </Show>
  );
}

/** Signed-out landing with Clerk sign-in button. */
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

