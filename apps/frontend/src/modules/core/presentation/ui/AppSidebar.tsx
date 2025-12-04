'use client';

import * as React from 'react';
import { AudioWaveform, Command, GalleryVerticalEnd, SquareTerminal } from 'lucide-react';

import { NavMain } from '@core/presentation/ui/NavMain';
import { TeamSwitcher } from '@core/presentation/ui/TeamSwitcher';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@shadcn/sidebar';
import { NavUser } from '@core/presentation/ui/NavUser.tsx';

// This is sample data.
const data = {
  user: {
    name: 'shadcn',
    email: 'm@example.com',
    avatar: '/avatars/shadcn.jpg',
  },
  teams: [
    {
      name: 'Acme Inc',
      logo: GalleryVerticalEnd,
      plan: 'Enterprise',
    },
    {
      name: 'Acme Corp.',
      logo: AudioWaveform,
      plan: 'Startup',
    },
    {
      name: 'Evil Corp.',
      logo: Command,
      plan: 'Free',
    },
  ],
  navMain: [
    {
      title: 'Pipelines',
      url: '#',
      icon: SquareTerminal,
      isActive: true,
      items: [
        {
          title: 'Pipelines',
          url: '/pipeline-creation',
        },
        {
          title: 'Marketplace',
          url: '/marketplace',
        },
      ],
    },

    {
      title: 'Settings',
      url: '#',
      icon: SquareTerminal,
      isActive: true,
      items: [
        {
          title: 'Users',
          url: '/user_settings',
        },
      ],
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible='icon' {...props}>
      <SidebarHeader>
        <TeamSwitcher teams={data.teams} />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
