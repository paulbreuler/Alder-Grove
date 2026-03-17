import { create } from 'zustand';

export type Theme = 'dark' | 'light' | 'system';

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

function applyTheme(theme: Theme): void {
  const resolved =
    theme === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
      : theme;
  const html = document.documentElement;
  html.classList.toggle('dark', resolved === 'dark');
  html.classList.toggle('light', resolved === 'light');
}

export const useThemeStore = create<ThemeState>()((set) => ({
  theme: 'dark',
  setTheme: (theme): void => {
    applyTheme(theme);
    set({ theme });
  },
}));
