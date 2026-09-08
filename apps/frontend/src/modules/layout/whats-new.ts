import { msg } from '@lingui/core/macro';
import type { MessageDescriptor } from '@lingui/core';
import type { LucideIcon } from 'lucide-react';
import { LayoutDashboard, Moon, ShieldIcon, UsersRound } from 'lucide-react';

/**
 * One thing worth telling the user about in a release.
 *
 * Declared as `msg` descriptors and Lucide components rather than JSON: the
 * copy has to go through Lingui like every other string, and an icon is a
 * component — a JSON file would need a translation table and a name→icon map
 * beside it to say the same thing.
 */
export interface ReleaseHighlight {
  /** Stable id — also the key of its "seen" flag in `localStorage`. */
  id: string;
  title: MessageDescriptor;
  description: MessageDescriptor;
  icon: LucideIcon;
  /**
   * The `NavEntry.url` this highlight leads to (no organization prefix). When
   * set, that sidebar entry carries a "New" badge until the user opens it.
   * Omit for anything that isn't a page.
   */
  navUrl?: string;
}

export interface Release {
  /** Bump it to re-arm every announcement: the seen flags are keyed by it. */
  version: string;
  highlights: readonly ReleaseHighlight[];
}

/**
 * THE release announcement. Editing this file is the whole job of announcing a
 * feature — the dialog on first launch and the sidebar badges are derived from
 * it, and both disappear on their own once the entries are gone.
 */
export const WHATS_NEW = {
  version: '0.4.0',
  highlights: [
    {
      id: 'dashboard',
      title: msg`Dashboard`,
      description: msg`Live metrics for your agents, pipelines and recent runs, all in one place.`,
      icon: LayoutDashboard,
      navUrl: 'dashboard',
    },
    {
      id: 'roles',
      title: msg`Roles`,
      description: msg`Build your own roles and grant them fine-grained permissions on an organization or a project.`,
      icon: ShieldIcon,
      navUrl: 'roles',
    },
    {
      id: 'members',
      title: msg`Members`,
      description: msg`Invite people to your organization and your projects, and manage what each of them can do.`,
      icon: UsersRound,
      navUrl: 'members',
    },
    {
      id: 'dark-mode',
      title: msg`Dark mode`,
      description: msg`Scylla now comes in light and dark — switch with the button in the top-right corner.`,
      icon: Moon,
    },
  ],
} satisfies Release;
