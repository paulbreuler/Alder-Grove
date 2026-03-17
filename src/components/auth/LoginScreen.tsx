import { useSignIn } from '@clerk/react';
import { invoke } from '@tauri-apps/api/core';
import { Apple, Github } from 'lucide-react';
import { useAuthStore } from '@/stores/useAuthStore';

type OAuthStrategy = 'oauth_apple' | 'oauth_github' | 'oauth_google';

const OAUTH_PROVIDERS: ReadonlyArray<{
  strategy: OAuthStrategy;
  label: string;
  Icon: typeof Apple | null;
}> = [
  { strategy: 'oauth_apple', label: 'Apple', Icon: Apple },
  { strategy: 'oauth_github', label: 'GitHub', Icon: Github },
  { strategy: 'oauth_google', label: 'Google', Icon: null },
];

export function LoginScreen(): React.JSX.Element {
  const { signIn, fetchStatus } = useSignIn();
  const status = useAuthStore((s) => s.status);
  const error = useAuthStore((s) => s.error);
  const setStatus = useAuthStore((s) => s.setStatus);
  const setError = useAuthStore((s) => s.setError);
  const clearError = useAuthStore((s) => s.clearError);

  const isLoading = status === 'authenticating' || fetchStatus === 'fetching';

  async function handleOAuth(strategy: OAuthStrategy): Promise<void> {
    if (!signIn) return;
    setStatus('authenticating');
    try {
      // Clerk v6 sso() initiates the OAuth redirect. In a Tauri webview,
      // the redirect will navigate to the provider's auth page.
      // The allowedRedirectProtocols on ClerkProvider permits grove:// callbacks.
      const result = await signIn.sso({
        strategy,
        redirectUrl: 'grove://callback',
        redirectCallbackUrl: 'grove://callback',
      });
      if (result.error) {
        setError(result.error.message ?? 'OAuth initiation failed');
      }
    } catch (err) {
      // If Clerk tries to navigate and we need to open the system browser instead,
      // catch the navigation attempt and use Tauri's open command
      const errStr = String(err);
      if (errStr.includes('http')) {
        try {
          await invoke('open_auth_url', { url: errStr });
        } catch (invokeErr) {
          setError(`Failed to open browser: ${String(invokeErr)}`);
        }
      } else {
        setError(errStr);
      }
    }
  }

  return (
    <div className="flex flex-col items-center justify-center h-full gap-[var(--grove-space-6)]">
      <div className="flex flex-col items-center gap-[var(--grove-space-2)]">
        <h1
          className="text-2xl font-semibold"
          style={{ color: 'var(--grove-text-primary)' }}
        >
          Alder Grove
        </h1>
        <p
          className="text-[var(--grove-font-size-sm)]"
          style={{ color: 'var(--grove-text-muted)' }}
        >
          Your applications grow in the Grove.
        </p>
      </div>

      {error && (
        <div
          className="max-w-sm px-[var(--grove-space-4)] py-[var(--grove-space-3)] text-[var(--grove-font-size-sm)] rounded-[var(--grove-radius-md)]"
          style={{
            backgroundColor: 'color-mix(in srgb, var(--grove-accent) 10%, transparent)',
            color: 'var(--grove-text-primary)',
          }}
        >
          <p>{error}</p>
          <button
            onClick={clearError}
            className="mt-[var(--grove-space-2)] text-xs underline hover:no-underline"
          >
            Dismiss
          </button>
        </div>
      )}

      <div className="flex flex-col gap-[var(--grove-space-3)] w-72">
        {OAUTH_PROVIDERS.map(({ strategy, label, Icon }) => (
          <button
            key={strategy}
            onClick={() => void handleOAuth(strategy)}
            disabled={isLoading}
            aria-label={`Sign in with ${label}`}
            className="flex items-center justify-center gap-[var(--grove-space-2)] w-full px-[var(--grove-space-4)] py-2.5 text-[var(--grove-font-size-sm)] font-medium rounded-[var(--grove-radius-md)] border transition-colors disabled:opacity-50"
            style={{
              borderColor: 'var(--grove-border-default)',
              backgroundColor: 'var(--grove-surface-elevated)',
              color: 'var(--grove-text-primary)',
            }}
          >
            {Icon && <Icon size={18} />}
            {isLoading ? 'Waiting for browser...' : `Sign in with ${label}`}
          </button>
        ))}
      </div>
    </div>
  );
}
