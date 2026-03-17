import type { Extension } from '@paulbreuler/shell';
import { homeExtension } from '@/extensions/home/extension';
import { userExtension } from '@/extensions/user/extension';
import { settingsExtension } from '@/extensions/settings/extension';

export const extensions: Extension[] = [
  homeExtension,
  userExtension,
  settingsExtension,
];
