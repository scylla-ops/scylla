import type { Snippet } from 'svelte';
import type { LucideIcon } from '@shared/presentation/ui/icon.ts';

export interface NavItem {
  title: string;
  url: string;
  icon?: LucideIcon;
  /**
   * The release highlight that this entry announces, if the current release has
   * one for its URL. See `layout/whats-new.ts`.
   */
  highlightId?: string;
}

export interface NavSection {
  title: string;
  items: NavItem[];
  /**
   * Shows at the top of the section card, in place of the section label. The
   * organization selector uses it.
   */
  header?: Snippet;
}
