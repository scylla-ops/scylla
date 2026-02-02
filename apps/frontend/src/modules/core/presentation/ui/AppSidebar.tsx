'use client';

import * as React from 'react';
import { AudioWaveform, Command, GalleryVerticalEnd, SquareTerminal } from 'lucide-react';

import { NavMain } from '@core/presentation/ui/NavMain';
import { OrganisationSwitcher } from '@core/presentation/ui/OrganisationSwitcher';
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
  organisations: [
    {
      name: 'Alpha',
      logo: GalleryVerticalEnd,
      description: 'Enterprise',
    },
    {
      name: 'Beta',
      logo: AudioWaveform,
      description: 'Startup',
    },
    {
      name: 'Zeta',
      logo: Command,
      description: 'Free',
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
          title: 'Create pipeline',
          url: '/pipeline-creation',
        },
        {
          title: 'Pipelines',
          url: '/pipeline-dashboard',
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
          url: '/user-settings',
        },
      ],
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible='icon' {...props}>
      <SidebarHeader>
        <OrganisationSwitcher organisations={data.organisations} />
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
