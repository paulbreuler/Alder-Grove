import { create } from 'zustand';
import type { SettingsTreeGroup } from './settingsTree';

interface SettingsStoreState {
  groups: SettingsTreeGroup[];
  selectedItemId: string | null;
  expandedGroups: string[];
  registerGroup: (group: SettingsTreeGroup) => void;
  unregisterGroup: (groupId: string) => void;
  selectItem: (id: string) => void;
  toggleGroup: (id: string) => void;
}

export const useSettingsStore = create<SettingsStoreState>()((set) => ({
  groups: [],
  selectedItemId: null,
  expandedGroups: [],

  registerGroup: (group): void => {
    set((s) => {
      const existing = s.groups.findIndex((g) => g.id === group.id);
      if (existing >= 0) {
        const updated = [...s.groups];
        updated[existing] = group;
        return { groups: updated };
      }
      const groups = [...s.groups, group];
      const expandedGroups =
        s.expandedGroups.length === 0 ? [group.id] : s.expandedGroups;
      const selectedItemId =
        s.selectedItemId === null && group.children.length > 0
          ? (group.children[0]?.id ?? null)
          : s.selectedItemId;
      return { groups, expandedGroups, selectedItemId };
    });
  },

  unregisterGroup: (groupId): void => {
    set((s) => {
      const groupToRemove = s.groups.find((g) => g.id === groupId);
      if (!groupToRemove) return s;
      const groups = s.groups.filter((g) => g.id !== groupId);
      const expandedGroups = s.expandedGroups.filter((gid) => gid !== groupId);
      const removedIds = new Set(
        groupToRemove.children.map((c) => c.id),
      );
      let selectedItemId = s.selectedItemId;
      if (selectedItemId && removedIds.has(selectedItemId)) {
        selectedItemId = groups[0]?.children[0]?.id ?? null;
      }
      return { groups, expandedGroups, selectedItemId };
    });
  },

  selectItem: (id): void => set({ selectedItemId: id }),

  toggleGroup: (id): void => {
    set((s) => ({
      expandedGroups: s.expandedGroups.includes(id)
        ? s.expandedGroups.filter((g) => g !== id)
        : [...s.expandedGroups, id],
    }));
  },
}));
