import { ChevronRight } from 'lucide-react';
import { cn } from '@paulbreuler/shell';
import { useSettingsStore } from './useSettingsStore';

export function SettingsLeftPanel(): React.JSX.Element {
  const groups = useSettingsStore((s) => s.groups);
  const selectedItemId = useSettingsStore((s) => s.selectedItemId);
  const expandedGroups = useSettingsStore((s) => s.expandedGroups);
  const selectItem = useSettingsStore((s) => s.selectItem);
  const toggleGroup = useSettingsStore((s) => s.toggleGroup);

  return (
    <nav className="flex flex-col py-2 text-sm">
      {groups.map((group) => {
        const isExpanded = expandedGroups.includes(group.id);
        const Icon = group.icon;
        return (
          <div key={group.id}>
            <button
              onClick={() => toggleGroup(group.id)}
              className="flex items-center gap-2 w-full px-3 py-1.5 hover:bg-[var(--grove-surface-elevated)] text-[var(--grove-text-primary)]"
            >
              <ChevronRight
                size={14}
                className={cn(
                  'transition-transform',
                  isExpanded && 'rotate-90',
                )}
              />
              <Icon size={16} />
              <span>{group.label}</span>
            </button>
            {isExpanded &&
              group.children.map((leaf) => (
                <button
                  key={leaf.id}
                  onClick={() => selectItem(leaf.id)}
                  className={cn(
                    'w-full text-left pl-10 pr-3 py-1.5 text-[var(--grove-text-secondary)] hover:bg-[var(--grove-surface-elevated)]',
                    selectedItemId === leaf.id &&
                      'bg-[var(--grove-surface-elevated)] text-[var(--grove-text-primary)]',
                  )}
                >
                  {leaf.label}
                </button>
              ))}
          </div>
        );
      })}
    </nav>
  );
}
