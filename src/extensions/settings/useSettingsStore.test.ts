import { describe, it, expect, beforeEach } from 'vitest';
import type { SettingsTreeGroup } from './settingsTree';
import { useSettingsStore } from './useSettingsStore';

/* ------------------------------------------------------------------ */
/*  Test fixtures                                                      */
/* ------------------------------------------------------------------ */

const StubComponent = (): null => null;
const StubIcon = (() => null) as unknown as SettingsTreeGroup['icon'];

function makeGroup(
  id: string,
  childIds: string[] = [`${id}-leaf`],
): SettingsTreeGroup {
  return {
    type: 'group',
    id,
    label: id.charAt(0).toUpperCase() + id.slice(1),
    icon: StubIcon,
    children: childIds.map((cid) => ({
      type: 'leaf' as const,
      id: cid,
      label: cid,
      component: StubComponent,
    })),
  };
}

/* ------------------------------------------------------------------ */
/*  Reset helper                                                       */
/* ------------------------------------------------------------------ */

function resetStore(): void {
  const { groups, unregisterGroup } = useSettingsStore.getState();
  for (const g of groups) {
    unregisterGroup(g.id);
  }
  // Ensure a clean slate (selection may linger after unregistering all)
  useSettingsStore.setState({
    groups: [],
    selectedItemId: null,
    expandedGroups: [],
  });
}

/* ------------------------------------------------------------------ */
/*  Tests                                                              */
/* ------------------------------------------------------------------ */

describe('useSettingsStore', () => {
  beforeEach(() => {
    resetStore();
  });

  /* ---- initial state -------------------------------------------- */

  it('starts with empty tree, no selection, no expansion', () => {
    const { groups, selectedItemId, expandedGroups } =
      useSettingsStore.getState();

    expect(groups).toEqual([]);
    expect(selectedItemId).toBeNull();
    expect(expandedGroups).toEqual([]);
  });

  /* ---- registerGroup -------------------------------------------- */

  describe('registerGroup', () => {
    it('adds a group and auto-selects first leaf child', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme', 'font']));

      const { groups, selectedItemId } = useSettingsStore.getState();
      expect(groups).toHaveLength(1);
      expect(groups[0].id).toBe('appearance');
      expect(selectedItemId).toBe('theme');
    });

    it('auto-expands the first registered group', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));

      const { expandedGroups } = useSettingsStore.getState();
      expect(expandedGroups).toContain('appearance');
    });

    it('does not override existing expansion when a second group is registered', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));
      registerGroup(makeGroup('editor'));

      const { expandedGroups } = useSettingsStore.getState();
      expect(expandedGroups).toEqual(['appearance']);
    });

    it('does not change selection when a second group is registered', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme']));
      registerGroup(makeGroup('editor', ['keybindings']));

      const { selectedItemId } = useSettingsStore.getState();
      expect(selectedItemId).toBe('theme');
    });

    it('does not duplicate when the same id is registered twice', () => {
      const { registerGroup } = useSettingsStore.getState();
      const group = makeGroup('appearance');
      registerGroup(group);
      registerGroup(group);

      const { groups } = useSettingsStore.getState();
      expect(groups).toHaveLength(1);
    });

    it('updates an existing group in place when re-registered with the same id', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme']));

      const updated = makeGroup('appearance', ['theme', 'font']);
      registerGroup(updated);

      const { groups } = useSettingsStore.getState();
      expect(groups).toHaveLength(1);
      expect(groups[0].children).toHaveLength(2);
    });

    it('handles a group with no children without crashing', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('empty', []));

      const { groups, selectedItemId } = useSettingsStore.getState();
      expect(groups).toHaveLength(1);
      expect(selectedItemId).toBeNull();
    });
  });

  /* ---- unregisterGroup ------------------------------------------ */

  describe('unregisterGroup', () => {
    it('removes a group from the tree', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));
      registerGroup(makeGroup('editor'));

      useSettingsStore.getState().unregisterGroup('appearance');

      const { groups } = useSettingsStore.getState();
      expect(groups).toHaveLength(1);
      expect(groups[0].id).toBe('editor');
    });

    it('clears selection when the selected leaf belonged to the removed group', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme']));

      // 'theme' is auto-selected
      expect(useSettingsStore.getState().selectedItemId).toBe('theme');

      useSettingsStore.getState().unregisterGroup('appearance');

      expect(useSettingsStore.getState().selectedItemId).toBeNull();
    });

    it('falls back selection to first leaf of remaining group', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme']));
      registerGroup(makeGroup('editor', ['keybindings']));

      // select a leaf in 'appearance'
      useSettingsStore.getState().selectItem('theme');
      useSettingsStore.getState().unregisterGroup('appearance');

      expect(useSettingsStore.getState().selectedItemId).toBe('keybindings');
    });

    it('preserves selection when an unrelated group is removed', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme']));
      registerGroup(makeGroup('editor', ['keybindings']));

      useSettingsStore.getState().selectItem('keybindings');
      useSettingsStore.getState().unregisterGroup('appearance');

      expect(useSettingsStore.getState().selectedItemId).toBe('keybindings');
    });

    it('removes the group from expandedGroups', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));

      expect(useSettingsStore.getState().expandedGroups).toContain('appearance');

      useSettingsStore.getState().unregisterGroup('appearance');

      expect(useSettingsStore.getState().expandedGroups).not.toContain(
        'appearance',
      );
    });

    it('is a no-op for an unknown group id', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));

      useSettingsStore.getState().unregisterGroup('nonexistent');

      expect(useSettingsStore.getState().groups).toHaveLength(1);
    });
  });

  /* ---- toggleGroup ---------------------------------------------- */

  describe('toggleGroup', () => {
    it('adds a group id to expandedGroups when collapsed', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));
      registerGroup(makeGroup('editor'));

      // 'editor' is not auto-expanded (only first group is)
      expect(useSettingsStore.getState().expandedGroups).not.toContain(
        'editor',
      );

      useSettingsStore.getState().toggleGroup('editor');

      expect(useSettingsStore.getState().expandedGroups).toContain('editor');
    });

    it('removes a group id from expandedGroups when expanded', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));

      expect(useSettingsStore.getState().expandedGroups).toContain(
        'appearance',
      );

      useSettingsStore.getState().toggleGroup('appearance');

      expect(useSettingsStore.getState().expandedGroups).not.toContain(
        'appearance',
      );
    });

    it('toggles back and forth correctly', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance'));

      useSettingsStore.getState().toggleGroup('appearance'); // collapse
      expect(useSettingsStore.getState().expandedGroups).not.toContain(
        'appearance',
      );

      useSettingsStore.getState().toggleGroup('appearance'); // expand
      expect(useSettingsStore.getState().expandedGroups).toContain(
        'appearance',
      );
    });
  });

  /* ---- selectItem ----------------------------------------------- */

  describe('selectItem', () => {
    it('sets selectedItemId', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme', 'font']));

      useSettingsStore.getState().selectItem('font');

      expect(useSettingsStore.getState().selectedItemId).toBe('font');
    });

    it('overwrites a previous selection', () => {
      const { registerGroup } = useSettingsStore.getState();
      registerGroup(makeGroup('appearance', ['theme', 'font']));

      useSettingsStore.getState().selectItem('font');
      useSettingsStore.getState().selectItem('theme');

      expect(useSettingsStore.getState().selectedItemId).toBe('theme');
    });
  });
});
