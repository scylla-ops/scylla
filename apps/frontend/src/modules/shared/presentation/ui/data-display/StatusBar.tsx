import { cn } from '@shared/presentation/utils';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import type { ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';

export interface StatusBarItem {
  id: string;
  status: string;
  tooltip?: ReactNode;
  /** Makes the segment activatable — it renders as a button instead of a plain bar. */
  onSelect?: () => void;
  /**
   * Accessible name for the activatable form. A colored bar has no text of its
   * own, and the tooltip is only `aria-describedby`, so without this the button
   * is unnameable for a screen reader and unfindable by name in a test.
   */
  label?: string;
}

interface StatusBarProps {
  items: StatusBarItem[];
  emptyLabel?: ReactNode;
  className?: string;
  height?: string;
}

/**
 * Generic bar chart showing colored segments per status.
 * Used for pipeline job history and job node timelines.
 */
export const StatusBar = ({ items, emptyLabel, className, height = 'h-6' }: StatusBarProps) => {
  if (items.length === 0) {
    return (
      <div className={cn('w-full flex items-center justify-center py-1', height)}>
        <span className='text-xs text-muted-foreground italic'>
          {emptyLabel ?? <Trans>No data</Trans>}
        </span>
      </div>
    );
  }

  return (
    <TooltipProvider delayDuration={100}>
      <div
        className={cn(
          'w-full flex items-center gap-[1px] py-1 overflow-hidden rounded-md',
          height,
          className,
        )}
      >
        {items.map(item => {
          const config = getStatusConfig(item.status);
          const className = cn(
            'flex-1 min-w-[2px] max-w-full h-full rounded-sm transition-all duration-150 shrink',
            config.barClassName,
            config.barHoverClassName,
            (item.tooltip || item.onSelect) && 'cursor-pointer',
          );

          const bar = item.onSelect ? (
            <button
              key={item.id}
              type='button'
              aria-label={item.label}
              onClick={event => {
                event.stopPropagation();
                item.onSelect?.();
              }}
              className={className}
            />
          ) : (
            <div key={item.id} className={className} />
          );

          if (!item.tooltip) return bar;

          return (
            <Tooltip key={item.id}>
              <TooltipTrigger asChild>{bar}</TooltipTrigger>
              <TooltipContent
                side='top'
                className='text-xs text-popover-foreground border-border p-3 shadow-lg bg-popover'
              >
                {item.tooltip}
              </TooltipContent>
            </Tooltip>
          );
        })}
      </div>
    </TooltipProvider>
  );
};
