import { useEffect, useRef } from 'react';
import { useSession } from '@clerk/react';
import { invoke } from '@tauri-apps/api/core';
import { useAuthStore } from '@/stores/useAuthStore';

/**
 * Syncs the Clerk session token to grove-tauri's encrypted store.
 *
 * Clerk refreshes short-lived JWTs (~60s) automatically. This hook
 * pushes each fresh token to Rust via IPC so the HTTP client can
 * make authenticated API calls.
 *
 * Only active when the Clerk session exists (user is signed in).
 */
export function useClerkSync(): void {
  const { session } = useSession();
  const { setStatus, setError } = useAuthStore();
  const syncedRef = useRef(false);

  useEffect(() => {
    if (!session) return;

    let cancelled = false;

    async function syncToken(): Promise<void> {
      try {
        const token = await session!.getToken();
        if (cancelled || !token) return;
        await invoke('set_clerk_token', { token });
        if (!syncedRef.current) {
          setStatus('ready');
          syncedRef.current = true;
        }
      } catch (err) {
        if (!cancelled) {
          setError(`Token sync failed: ${String(err)}`);
        }
      }
    }

    // Initial sync
    void syncToken();

    // Re-sync on token refresh (Clerk fires this every ~60s)
    const interval = setInterval(() => void syncToken(), 50_000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [session, setStatus, setError]);
}
