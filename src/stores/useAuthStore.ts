import { create } from 'zustand';

export type AuthStatus = 'idle' | 'authenticating' | 'ready' | 'error';

interface AuthState {
  status: AuthStatus;
  error: string | null;
  setStatus: (status: AuthStatus) => void;
  setError: (error: string) => void;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>()((set) => ({
  status: 'idle',
  error: null,

  setStatus: (status): void => set({ status, error: null }),

  setError: (error): void => set({ status: 'error', error }),

  clearError: (): void => set({ status: 'idle', error: null }),
}));
