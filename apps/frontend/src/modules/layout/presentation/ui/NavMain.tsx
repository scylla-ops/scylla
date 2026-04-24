import { Trans } from '@lingui/react/macro';
import { type LucideIcon } from 'lucide-react';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { useScyllaNavigate } from '@/modules/shared/presentation/hooks/useScyllaNavigate';

export function NavMain({
  items,
}: {
  items: {
    title: string;
    url: string;
    icon?: LucideIcon;
    isActive?: boolean;
    disabled?: boolean;
  }[];
}) {
  const navigate = useScyllaNavigate().navigate;

  return (
    <SidebarGroup>
      <SidebarGroupLabel>
        <Trans>Main</Trans>
      </SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item, index) => (
          <SidebarMenuItem
            className={
              item.disabled
                ? 'pointer-events-none opacity-50'
                : 'transition-all duration-200 hover:scale-105'
            }
            key={index}
          >
            <SidebarMenuButton
              tooltip={item.title}
              disabled={item.disabled}
              aria-disabled={item.disabled}
              onClick={() => {
                if (item.disabled) return;
                navigate(item.url);
              }}
            >
              {item.icon && <item.icon />}
              <span>{item.title}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}
