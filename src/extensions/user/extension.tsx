import type { Extension } from '@paulbreuler/shell';
import { UserAvatar } from './UserAvatar';

export const userExtension: Extension = {
  id: 'grove.user',
  name: 'User Profile',

  activate: (ctx): void => {
    // Register in bottom zone, above Settings (order 100)
    ctx.registerActivityBarItem({
      id: 'grove.user',
      icon: UserAvatar,
      label: 'Profile',
      order: 90,
      zone: 'bottom',
    });
  },
};
