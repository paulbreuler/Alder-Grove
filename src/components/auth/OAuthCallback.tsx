import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useClerk } from '@clerk/react';
import { useAuthStore } from '@/stores/useAuthStore';

/**
 * Listens for the 'auth-callback' event emitted by grove-tauri's one-shot
 * HTTP server after the system browser completes OAuth and redirects to
 * http://127.0.0.1:19287/clerk-callback?...
 *
 * On receiving the callback URL, navigates the webview to the SSO callback
 * path so Clerk's JS SDK can complete the session.
 */
export function OAuthCallback(): null {
  const clerk = useClerk();
  const { setError, setStatus } = useAuthStore();

  useEffect(() => {
    const unlistenPromise = listen<string>('auth-callback', async (event) => {
      try {
        const callbackUrl = event.payload.replace(/^"+|"+$/g, '');
        const url = new URL(callbackUrl);
        const params = url.search;

        if (params) {
          // Clerk needs the callback params in the webview URL to complete
          // the sign-in. Use hash routing to avoid a full page reload.
          window.location.hash = `#/sso-callback${params}`;

          // Attempt to handle the redirect callback via Clerk SDK
          try {
            await clerk.handleRedirectCallback(
              {} as Parameters<typeof clerk.handleRedirectCallback>[0],
            );
          } catch {
            // Clerk may handle this through its own URL watching —
            // the hash change above is often sufficient.
          }
        }
      } catch (err) {
        setError(`OAuth callback failed: ${String(err)}`);
        setStatus('idle');
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [clerk, setError, setStatus]);

  return null;
}
