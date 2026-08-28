import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import type {
  NavItem,
  NavSection,
} from '@/modules/layout/presentation/structs/nav-section.struct.ts';
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
import { cn } from '@shared/presentation/utils/cn.ts';

/**
 * Each nav section reads as a self-contained card floating on the sidebar
 * background. Collapsed to the icon rail the card shrinks to hug the 2rem icon
 * column (centered in the 3rem rail) rather than disappearing — the grouping
 * is the whole point, and it survives the collapse.
 */
export const NAV_SECTION_CARD_CLASS = cn(
  'gap-1 rounded-xl border border-border bg-background p-2 shadow-sm',
  'group-data-[collapsible=icon]:mx-auto group-data-[collapsible=icon]:w-fit',
  'group-data-[collapsible=icon]:rounded-lg group-data-[collapsible=icon]:p-1',
);

/** Hairline between a section's header and its entries, inside the card. */
export const NAV_SECTION_DIVIDER_CLASS = 'my-1 border-t border-border';

interface NavMainProps {
  sections: NavSection[];
}

export function NavMain({ sections }: NavMainProps) {
  const navigate = useScyllaNavigate().navigate;
  const { pathname } = useLocation();

  const isActive = (url?: string) => !!url && pathname.startsWith(url);

  const renderLeaf = (item: NavItem) => (
    <SidebarMenuItem
      className={
        // The lift would push the icon past the card edge on the collapsed rail.
        'transition-all duration-200 hover:scale-105 group-data-[collapsible=icon]:hover:scale-100'
      }
    >
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
        <SidebarGroup key={index} className={NAV_SECTION_CARD_CLASS}>
          {section.header ? (
            <>
              {section.header}
              <div className={NAV_SECTION_DIVIDER_CLASS} />
            </>
          ) : (
            // Fully removed (not just faded) on the rail: the card is sized to
            // its content, and a laid-out label would stretch it to the title.
            <SidebarGroupLabel className='group-data-[collapsible=icon]:hidden'>
              <Trans>{section.title}</Trans>
            </SidebarGroupLabel>
          )}
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
