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
  /**
   * The action is running. Marks the control `aria-busy` and disables it, so
   * "busy" is one prop rather than a `disabled` and a spinning icon that can
   * drift apart.
   */
  busy?: boolean;
};

/**
 * A reusable icon button wrapped in a tooltip.
 * Default style matches the common pattern used across the app (ghost, rounded-full, hover scale).
 *
 * The tooltip text is also rendered visually-hidden inside the button: a
 * tooltip is only wired up as `aria-describedby`, and while it is closed that
 * leaves the button with no accessible name at all — unusable by a screen
 * reader, and unfindable by name in a test.
 */
export const IconButton = ({
  icon: Icon,
  tooltip,
  onClick,
  className,
  iconClassName,
  disabled,
  busy,
}: IconButtonProps) => {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          size='icon'
          variant='ghost'
          disabled={disabled || busy}
          aria-busy={busy || undefined}
          className={cn(
            'h-8 w-8 cursor-pointer rounded-full transition-all duration-200 hover:scale-125 hover:bg-primary-subtle hover:text-primary active:scale-95',
            className,
          )}
          onClick={onClick}
        >
          <Icon className={cn('h-4 w-4', iconClassName)} />
          <span className='sr-only'>{tooltip}</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>{tooltip}</p>
      </TooltipContent>
    </Tooltip>
  );
};
