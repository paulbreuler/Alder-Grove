import { Settings, Palette } from 'lucide-react';
import type { Extension } from '@paulbreuler/shell';
import { SettingsLeftPanel } from './SettingsLeftPanel';
import { SettingsEditorView } from './SettingsEditorView';
import { ThemeSettings } from './ThemeSettings';
import { useSettingsStore } from './useSettingsStore';

const SettingsIcon = ({
  isActive,
}: {
  isActive: boolean;
}): React.JSX.Element => (
  <Settings size={20} strokeWidth={isActive ? 2 : 1.5} />
);

export const settingsExtension: Extension = {
  id: 'grove.settings',
  name: 'Settings',

  activate: (ctx): void => {
    ctx.registerActivityBarItem({
      id: 'grove.settings',
      icon: SettingsIcon,
      label: 'Settings',
      order: 100,
      zone: 'utility',
    });

    ctx.registerLeftPanelView({
      id: 'grove.settings.left-panel',
      parentActivity: 'grove.settings',
      component: SettingsLeftPanel,
      title: 'Settings',
    });

    ctx.registerEditorView({
      id: 'grove.settings.editor',
      parentActivity: 'grove.settings',
      component: SettingsEditorView,
      title: 'Settings',
    });

    useSettingsStore.getState().registerGroup({
      type: 'group',
      id: 'appearance',
      label: 'Appearance',
      icon: Palette,
      children: [
        {
          type: 'leaf',
          id: 'appearance.theme',
          label: 'Theme & Display',
          component: ThemeSettings,
        },
      ],
    });
  },

  deactivate(): void {
    useSettingsStore.getState().unregisterGroup('appearance');
  },
};
