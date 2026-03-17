import type React from 'react';
import type { LucideIcon } from 'lucide-react';

export interface SettingsTreeLeaf {
  type: 'leaf';
  id: string;
  label: string;
  component: React.ComponentType;
}

export interface SettingsTreeGroup {
  type: 'group';
  id: string;
  label: string;
  icon: LucideIcon;
  children: SettingsTreeLeaf[];
}
