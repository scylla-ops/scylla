import type { LucideIcon } from 'lucide-react';
import type { Permission } from '@/modules/features/role/domain/structs/permission.struct.ts';

export interface NavItem {
  title: string;
  /** Destination route. Optional for parent items that only group sub-scopes. */
  url?: string;
  icon?: LucideIcon;
  isActive?: boolean;
  /**
   * Permission required (in the current org context) for this entry to be shown.
   * Omit for entries everyone may see.
   */
  permission?: Permission;
  /** Sub-scope entries rendered as a collapsible Pangolin-style sub-menu. */
  items?: NavItem[];
}

export interface NavSection {
  title: string;
  items: NavItem[];
}
