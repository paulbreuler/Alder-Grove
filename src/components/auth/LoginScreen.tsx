import { useSignIn } from '@clerk/react';
import { invoke } from '@tauri-apps/api/core';
import { Apple, Github } from 'lucide-react';
import { useAuthStore } from '@/stores/useAuthStore';

const OAUTH_PROVIDERS = [
  { strategy: 'oauth_apple' as const, label: 'Apple', Icon: Apple },
  { strategy: 'oauth_github' as const, label: 'GitHub', Icon: Github },
  { strategy: 'oauth_google' as const, label: 'Google', Icon: null },
] as const;

export function LoginScreen(): React.JSX.Element {
  const { signIn, isLoaded } = useSignIn();
  const status = useAuthStore((s) => s.status);
  const error = useAuthStore((s) => s.error);
  const setStatus = useAuthStore((s) => s.setStatus);
  const setError = useAuthStore((s) => s.setError);
  const clearError = useAuthStore((s) => s.clearError);

  const isLoading = status === 'authenticating';

  async function handleOAuth(strategy: string): Promise<void> {
    if (!signIn || !isLoaded) return;
    setStatus('authenticating');
    try {
      const result = await signIn.authenticateWithRedirect({
        strategy: strategy as 'oauth_apple',
        redirectUrl: 'grove://callback',
        redirectUrlComplete: 'grove://callback',
      });
      // Open the Clerk authorization URL in the system browser
      if (result?.externalVerificationRedirectURL) {
        await invoke('open_auth_url', {
          url: result.externalVerificationRedirectURL.toString(),
        });
      }
    } catch (err) {
      setError(String(err));
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
