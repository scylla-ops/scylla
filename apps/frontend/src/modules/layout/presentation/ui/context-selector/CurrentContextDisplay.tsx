import { ChevronsUpDown, type LucideIcon } from 'lucide-react';

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
  const bgClass =
    variant === 'primary'
      ? 'bg-sidebar-primary text-sidebar-primary-foreground'
      : 'bg-primary text-sidebar-primary-foreground';

  return (
    <>
      <div
        className={`${bgClass} flex aspect-square size-8 items-center justify-center rounded-lg`}
      >
        <Icon className='size-4' />
      </div>
      <div className='grid flex-1 text-left text-sm leading-tight'>
        <span className='truncate font-medium'>{name}</span>
        <span className='truncate text-xs text-muted-foreground'>{description}</span>
      </div>
      <ChevronsUpDown className='ml-auto' />
    </>
  );
};

