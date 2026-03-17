import type { LucideIcon } from 'lucide-react';

type ContextItemProps = {
  name: string;
  icon: LucideIcon;
};

export const ContextItem = ({ name, icon: Icon }: ContextItemProps) => {
  return (
    <div className='flex items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-sidebar-accent'>
      <div className='flex size-6 items-center justify-center rounded-md border'>
        <Icon className='size-3.5 shrink-0' />
      </div>
      <span className='flex-1 truncate'>{name}</span>
    </div>
  );
};

