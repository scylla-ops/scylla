import type { Snippet } from 'svelte';
import type { LucideIcon } from '@shared/presentation/ui/icon.ts';

export interface NavItem {
  title: string;
  url: string;
  icon?: LucideIcon;
  /** See `layout/whats-new.ts`. */
  highlightId?: string;
}

export interface NavSection {
  title: string;
  items: NavItem[];
  /** Replaces the section label (the organization selector). */
  header?: Snippet;
}
