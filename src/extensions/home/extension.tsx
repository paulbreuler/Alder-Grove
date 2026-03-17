import { Home } from 'lucide-react';
import type { Extension } from '@paulbreuler/shell';
import { HomeEditor } from './HomeEditor';

const HomeIcon = ({
  isActive,
}: {
  isActive: boolean;
}): React.JSX.Element => (
  <Home size={20} strokeWidth={isActive ? 2 : 1.5} />
);

export const homeExtension: Extension = {
  id: 'grove.home',
  name: 'Home',

  activate: (ctx): void => {
    ctx.registerActivityBarItem({
      id: 'grove.home',
      icon: HomeIcon,
      label: 'Home',
      order: 0,
      zone: 'activity-bar',
    });

    ctx.registerEditorView({
      id: 'grove.home.editor',
      parentActivity: 'grove.home',
      component: HomeEditor,
      title: 'Home',
    });
  },
};
