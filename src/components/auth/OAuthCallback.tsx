import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useAuthStore } from '@/stores/useAuthStore';

/**
 * Listens for the 'auth-callback' event emitted by grove-tauri when the
 * system browser redirects to grove://callback after OAuth.
 *
 * Clerk's JS SDK completes the session automatically when the webview URL
 * contains the callback parameters. This component handles the deep-link
 * bridge: it receives the callback URL from Tauri and updates window.location
 * so Clerk can process it.
 */
export function OAuthCallback(): null {
  const { setError } = useAuthStore();

  useEffect(() => {
    const unlistenPromise = listen<string>('auth-callback', (event) => {
      try {
        const callbackUrl = event.payload.replace(/^"+|"+$/g, '');
        // Extract query params from grove://callback?... and apply to current location
        const url = new URL(callbackUrl.replace(/^grove:\/\//, 'https://grove.local/'));
        const params = url.search;
        if (params) {
          // Navigate to the callback path so Clerk's redirect handler can pick it up
          window.location.hash = `/sso-callback${params}`;
        }
      } catch (err) {
        setError(`OAuth callback failed: ${String(err)}`);
      }
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [setError]);

  return null;
}
