import type { ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';
import type { Permission } from '@platform/authz';

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
  /**
   * Release highlight this entry announces, when the current release has one
   * for its url — see `layout/whats-new.ts`. Resolved by the shell, where the
   * unprefixed url is still known; the badge itself lives in `NavMain`.
   */
  highlightId?: string;
  /** Sub-scope entries rendered as a collapsible Pangolin-style sub-menu. */
  items?: NavItem[];
}

export interface NavSection {
  title: string;
  items: NavItem[];
  /**
   * Optional node pinned at the top of the section card (above its entries),
   * for controls that belong to the section itself — e.g. the organization
   * selector on top of the organization-scoped links.
   * Rendering it replaces the section label, which it already spells out.
   */
  header?: ReactNode;
}
