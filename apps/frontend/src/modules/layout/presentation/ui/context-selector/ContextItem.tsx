import type { LucideIcon } from 'lucide-react';

type ContextItemProps = {
  name: string;
  icon: LucideIcon;
};

export const ContextItem = ({ name, icon: Icon }: ContextItemProps) => {
  return (
    <div className='w-full group flex items-center gap-3 rounded-lg px-2 py-2 text-sm transition-colors cursor-pointer hover:bg-slate-50'>
      <div className='flex size-8 items-center justify-center rounded-md border border-slate-200 bg-white transition-colors group-hover:border-primary/50'>
        <Icon className='size-4 text-slate-500 transition-colors group-hover:text-primary' />
      </div>

      <span className='flex-1 truncate text-slate-700 font-medium transition-colors group-hover:text-primary'>
        {name}
      </span>
    </div>
  );
};
