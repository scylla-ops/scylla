import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import type {
  NavItem,
  NavSectionModel,
} from '@/modules/layout/presentation/models/nav-section.model.ts';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '@shadcn/sidebar.tsx';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@shadcn/collapsible.tsx';
import { ChevronRight } from 'lucide-react';
import { useLocation } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';

interface NavMainProps {
  sections: NavSectionModel[];
}

export function NavMain({ sections }: NavMainProps) {
  const navigate = useScyllaNavigate().navigate;
  const { pathname } = useLocation();

  const isActive = (url?: string) => !!url && pathname.startsWith(url);

  const renderLeaf = (item: NavItem) => (
    <SidebarMenuItem className={'transition-all duration-200 hover:scale-105'}>
      <SidebarMenuButton
        tooltip={item.title}
        isActive={isActive(item.url)}
        onClick={() => item.url && navigate(item.url)}
      >
        {item.icon && <item.icon />}
        <span>{item.title}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );

  const renderParent = (item: NavItem) => {
    const childActive = item.items?.some(sub => isActive(sub.url));

    return (
      <Collapsible asChild defaultOpen={childActive} className='group/collapsible'>
        <SidebarMenuItem>
          <CollapsibleTrigger asChild>
            <SidebarMenuButton tooltip={item.title}>
              {item.icon && <item.icon />}
              <span>{item.title}</span>
              <ChevronRight className='ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90' />
            </SidebarMenuButton>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <SidebarMenuSub>
              {item.items?.map((sub, subIndex) => (
                <SidebarMenuSubItem key={subIndex}>
                  <SidebarMenuSubButton
                    isActive={isActive(sub.url)}
                    onClick={() => sub.url && navigate(sub.url)}
                    className='cursor-pointer'
                  >
                    {sub.icon && <sub.icon />}
                    <span>{sub.title}</span>
                  </SidebarMenuSubButton>
                </SidebarMenuSubItem>
              ))}
            </SidebarMenuSub>
          </CollapsibleContent>
        </SidebarMenuItem>
      </Collapsible>
    );
  };

  return (
    <>
      {sections.map((section, index) => (
        <SidebarGroup key={index}>
          <SidebarGroupLabel>
            <Trans>{section.title}</Trans>
          </SidebarGroupLabel>
          <SidebarMenu>
            {section.items.map((item, itemIndex) => (
              <div key={itemIndex}>
                {item.items?.length ? renderParent(item) : renderLeaf(item)}
              </div>
            ))}
          </SidebarMenu>
        </SidebarGroup>
      ))}
    </>
  );
}
