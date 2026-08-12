import { ChevronsUpDown, type LucideIcon } from 'lucide-react';
import { useSidebar } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';

type CurrentContextDisplayProps = {
  name: string;
  description: string;
  icon: LucideIcon;
  variant?: 'primary' | 'secondary';
};

export const CurrentContextDisplay = ({
  name,
  description,
  icon: Icon,
  variant = 'primary',
}: CurrentContextDisplayProps) => {
  const { state } = useSidebar();
  const isCollapsed = state === 'collapsed';

  if (isCollapsed) {
    return (
      <div
        className={`
          flex items-center justify-center rounded-lg size-9
          cursor-pointer
          shadow-sm
          ${variant === 'primary' ? 'bg-primary text-primary-foreground' : 'bg-gradient-to-br from-slate-500 to-slate-600 text-white'}
        `}
      >
        <Icon className='size-4' strokeWidth={2.5} />
      </div>
    );
  }

  return (
    <>
      <div
        className={`
          flex items-center justify-center rounded-lg size-9 shrink-0
          shadow-sm
          ${variant === 'primary' ? 'bg-primary text-primary-foreground' : 'bg-gradient-to-br from-slate-500 to-slate-600 text-white'}
        `}
      >
        <Icon className='size-4' strokeWidth={2.5} />
      </div>

      <div className='flex-1 min-w-0 text-left'>
        <p className='text-sm font-semibold text-slate-900 dark:text-slate-100 truncate leading-tight'>
          {name}
        </p>
        <p className='text-xs text-slate-500 dark:text-slate-400 truncate leading-tight'>
          {description}
        </p>
      </div>

      <ChevronsUpDown className='size-4 text-muted-foreground shrink-0 ml-auto' />
    </>
  );
};
