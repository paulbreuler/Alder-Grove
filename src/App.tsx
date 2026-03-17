import { Show, SignInButton } from '@clerk/react';
import { Workbench } from '@paulbreuler/shell';

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
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100%',
        gap: 'var(--grove-space-lg)',
        background: 'var(--grove-bg-primary)',
        color: 'var(--grove-text-primary)',
        fontFamily: 'var(--grove-font-sans)',
      }}
    >
      <h1 style={{ margin: 0 }}>Alder Grove</h1>
      <p style={{ color: 'var(--grove-text-secondary)', margin: 0 }}>
        Your applications grow in the Grove.
      </p>
      <SignInButton mode="modal">
        <button
          style={{
            padding: `var(--grove-space-sm) var(--grove-space-lg)`,
            background: 'var(--grove-accent)',
            color: 'var(--grove-text-primary)',
            border: 'none',
            borderRadius: 'var(--grove-radius-md)',
            cursor: 'pointer',
            fontFamily: 'var(--grove-font-sans)',
            fontSize: 'var(--grove-font-size-base)',
          }}
        >
          Sign in
        </button>
      </SignInButton>
    </div>
  );
}

