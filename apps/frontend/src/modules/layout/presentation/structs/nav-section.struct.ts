import type { LucideIcon } from 'lucide-react';

export interface NavItem {
  title: string;
  /** Destination route. Optional for parent items that only group sub-scopes. */
  url?: string;
  icon?: LucideIcon;
  isActive?: boolean;
  /** Sub-scope entries rendered as a collapsible Pangolin-style sub-menu. */
  items?: NavItem[];
}

export interface NavSection {
  title: string;
  items: NavItem[];
}
