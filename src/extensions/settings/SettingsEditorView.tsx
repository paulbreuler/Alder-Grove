import { useSettingsStore } from './useSettingsStore';

export function SettingsEditorView(): React.JSX.Element {
  const groups = useSettingsStore((s) => s.groups);
  const selectedItemId = useSettingsStore((s) => s.selectedItemId);

  const selectedLeaf = groups
    .flatMap((g) => g.children)
    .find((leaf) => leaf.id === selectedItemId);

  if (!selectedLeaf) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--grove-text-secondary)]">
        Select a setting
      </div>
    );
  }

  const Component = selectedLeaf.component;
  return <Component />;
}
