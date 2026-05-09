import { cn } from '@shared/presentation/utils';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@shadcn/tooltip.tsx';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import type { ReactNode } from 'react';

export interface StatusBarItem {
  id: string;
  status: string;
  tooltip?: ReactNode;
}

interface StatusBarProps {
  items: StatusBarItem[];
  emptyLabel?: string;
  className?: string;
  height?: string;
}

/**
 * Generic bar chart showing colored segments per status.
 * Used for pipeline job history and job node timelines.
 */
export const StatusBar = ({ items, emptyLabel = 'No data', className, height = 'h-6' }: StatusBarProps) => {
  if (items.length === 0) {
    return (
      <div className={cn('w-full flex items-center justify-center py-1', height)}>
        <span className='text-xs text-slate-400 italic'>{emptyLabel}</span>
      </div>
    );
  }

  return (
    <TooltipProvider delayDuration={100}>
      <div className={cn('w-full flex items-center gap-1 py-1 overflow-hidden rounded-md', height, className)}>
        {items.map(item => {
          const config = getStatusConfig(item.status);

          const bar = (
            <div
              key={item.id}
              className={cn(
                'flex-1 min-w-[4px] h-full rounded-sm transition-all duration-150',
                config.barClassName,
                config.barHoverClassName,
                item.tooltip && 'cursor-help',
              )}
            />
          );

          if (!item.tooltip) return bar;

          return (
            <Tooltip key={item.id}>
              <TooltipTrigger asChild>{bar}</TooltipTrigger>
              <TooltipContent side='top' className='text-xs p-3 shadow-lg border-slate-200'>
                {item.tooltip}
              </TooltipContent>
            </Tooltip>
          );
        })}
      </div>
    </TooltipProvider>
  );
};

