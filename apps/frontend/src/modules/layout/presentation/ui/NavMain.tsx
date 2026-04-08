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
  }[];
}) {
  const navigate = useScyllaNavigate().navigate;

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Main</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item, index) => (
          <SidebarMenuItem className={'transition-all duration-200 hover:scale-105'} key={index}>
            <SidebarMenuButton tooltip={item.title} onClick={() => navigate(item.url)}>
              {item.icon && <item.icon />}
              <span>{item.title}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}
