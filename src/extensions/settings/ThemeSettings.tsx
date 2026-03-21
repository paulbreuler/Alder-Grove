import { useThemeStore } from '@paulbreuler/shell';

export function ThemeSettings(): React.JSX.Element {
  const themes = useThemeStore((s) => s.themes);
  const activeThemeId = useThemeStore((s) => s.activeThemeId);
  const activateTheme = useThemeStore((s) => s.activateTheme);

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
            value={activeThemeId}
            onChange={(e) => activateTheme(e.target.value)}
            className="text-sm bg-[var(--grove-surface-elevated)] text-[var(--grove-text-primary)] border border-[var(--grove-border-default)] rounded px-2 py-1"
          >
            {themes.map((t) => (
              <option key={t.id} value={t.id}>
                {t.name}
              </option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
}
