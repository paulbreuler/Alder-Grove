import { useThemeStore, type Theme } from '@/stores/useThemeStore';

const themes: { value: Theme; label: string }[] = [
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
  { value: 'system', label: 'System' },
];

export function ThemeSettings(): React.JSX.Element {
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);

  return (
    <div className="flex flex-col gap-6 p-6">
      <div>
        <h2 className="text-base font-semibold text-[var(--grove-text-primary)] mb-4">
          Appearance
        </h2>
        <div className="flex items-center justify-between py-3">
          <div className="flex flex-col gap-0.5">
            <span className="text-sm font-medium text-[var(--grove-text-primary)]">
              Theme
            </span>
            <span className="text-xs text-[var(--grove-text-secondary)]">
              Choose your preferred color scheme
            </span>
          </div>
          <select
            value={theme}
            onChange={(e) => setTheme(e.target.value as Theme)}
            className="text-sm bg-[var(--grove-surface-elevated)] text-[var(--grove-text-primary)] border border-[var(--grove-border-default)] rounded px-2 py-1"
          >
            {themes.map((t) => (
              <option key={t.value} value={t.value}>
                {t.label}
              </option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
}
