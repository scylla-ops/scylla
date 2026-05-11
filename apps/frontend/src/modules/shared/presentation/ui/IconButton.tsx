import type { LucideIcon } from 'lucide-react';
import type { ReactNode, SyntheticEvent } from 'react';
import { Button } from '@shadcn';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { cn } from '@shared/presentation/utils';

type IconButtonProps = {
  icon: LucideIcon;
  tooltip: ReactNode;
  onClick?: (e: SyntheticEvent) => void;
  className?: string;
  iconClassName?: string;
  disabled?: boolean;
};

/**
 * A reusable icon button wrapped in a tooltip.
 * Default style matches the common pattern used across the app (ghost, rounded-full, hover scale).
 */
export const IconButton = ({
  icon: Icon,
  tooltip,
  onClick,
  className,
  iconClassName,
  disabled,
}: IconButtonProps) => {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          size='icon'
          variant='ghost'
          disabled={disabled}
          className={cn(
            'h-8 w-8 cursor-pointer transition-all hover:scale-125 hover:text-primary hover:bg-primary-hover rounded-full',
            className,
          )}
          onClick={onClick}
        >
          <Icon className={cn('w-4 h-4', iconClassName)} />
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>{tooltip}</p>
      </TooltipContent>
    </Tooltip>
  );
};

