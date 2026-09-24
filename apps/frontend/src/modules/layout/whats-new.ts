import { msg } from '@lingui/core/macro';
import type { MessageDescriptor } from '@lingui/core';
import LayoutDashboard from '@lucide/svelte/icons/layout-dashboard';
import Moon from '@lucide/svelte/icons/moon';
import ShieldIcon from '@lucide/svelte/icons/shield';
import UsersRound from '@lucide/svelte/icons/users-round';
import type { LucideIcon } from '@shared/presentation/ui/icon.ts';

export interface ReleaseHighlight {
  /** Also the key of its "seen" flag in `localStorage`. */
  id: string;
  title: MessageDescriptor;
  description: MessageDescriptor;
  icon: LucideIcon;
  /** The sidebar entry that shows a "New" badge until the user opens it. */
  navUrl?: string;
}

export interface Release {
  /** Bump it to show every announcement again. */
  version: string;
  highlights: readonly ReleaseHighlight[];
}

/** The release announcement: the first-launch dialog and the sidebar badges come from it. */
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
