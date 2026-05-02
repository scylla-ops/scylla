import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import type { NavSectionModel } from '@/modules/layout/presentation/models/nav-section.model.ts';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@shadcn/sidebar.tsx';
import { Trans } from '@lingui/react/macro';

interface NavMainProps {
  sections: NavSectionModel[];
}

export function NavMain({ sections }: NavMainProps) {
  const navigate = useScyllaNavigate().navigate;

  return (
    <>
      {sections.map((section, index) => (
        <SidebarGroup key={index}>
          <SidebarGroupLabel>
            <Trans>{section.title}</Trans>
          </SidebarGroupLabel>
          <SidebarMenu>
            {section.items.map((item, index) => (
              <SidebarMenuItem
                className={'transition-all duration-200 hover:scale-105'}
                key={index}
              >
                <SidebarMenuButton tooltip={item.title} onClick={() => navigate(item.url)}>
                  {item.icon && <item.icon />}
                  <span>{item.title}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            ))}
          </SidebarMenu>
        </SidebarGroup>
      ))}
    </>
  );
}
