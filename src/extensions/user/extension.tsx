import type { Extension } from '@paulbreuler/shell';
import { UserAvatar } from './UserAvatar';

export const userExtension: Extension = {
  id: 'grove.user',
  name: 'User Profile',

  activate: (ctx): void => {
    // Bottom zone, above Settings. No editor/panel views —
    // SignInButton opens a modal, UserButton opens its own popup.
    ctx.registerActivityBarItem({
      id: 'grove.user',
      icon: UserAvatar,
      label: 'Profile',
      order: 90,
      zone: 'bottom',
    });
  },
};
