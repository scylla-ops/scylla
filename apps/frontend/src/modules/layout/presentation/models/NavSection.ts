import type { LucideIcon } from 'lucide-react';

export interface NavItem {
  title: string;
  url: string;
  icon?: LucideIcon;
  isActive?: boolean;
}

export interface NavSection {
  title: string;
  items: NavItem[];
}
