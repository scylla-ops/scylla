import React from 'react';
import { cn } from '@shared/presentation/utils';

export type StatusState =
  | 'success'
  | 'failed'
  | 'running'
  | 'pending'
  | 'idle'
  | 'skipped'
  | 'cancelled'
  | 'orphaned';

interface StatusIndicatorProps {
  state: StatusState;
  label?: string;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  labelClassName?: string;
  animateAllStates?: boolean;
}

const getStateColors = (state: StatusIndicatorProps['state']) => {
  switch (state) {
    case 'success':
      return {
        dot: 'bg-green-500',
        ping: 'bg-green-300',
        container: 'border-green-200 text-green-800 dark:border-green-800 dark:text-green-300',
        gradient: 'from-green-400 to-green-500',
      };
    case 'failed':
      return {
        dot: 'bg-red-500',
        ping: 'bg-red-300',
        container: 'border-red-200 text-red-800 dark:border-red-800 dark:text-red-300',
        gradient: 'from-red-400 to-red-500',
      };
    case 'running':
      return {
        dot: 'bg-blue-500',
        ping: 'bg-blue-300',
        container: 'border-blue-200 text-blue-800 dark:border-blue-800 dark:text-blue-300',
        gradient: 'from-blue-400 to-blue-500',
      };
    case 'skipped':
      return {
        dot: 'bg-zinc-500',
        ping: 'bg-zinc-300',
        container: 'border-zinc-200 text-zinc-700 dark:border-zinc-700 dark:text-zinc-300',
        gradient: 'from-zinc-400 to-zinc-500',
      };
    case 'cancelled':
      return {
        dot: 'bg-amber-500',
        ping: 'bg-amber-300',
        container: 'border-amber-200 text-amber-800 dark:border-amber-800 dark:text-amber-300',
        gradient: 'from-amber-400 to-amber-500',
      };
    case 'orphaned':
      return {
        dot: 'bg-orange-500',
        ping: 'bg-orange-300',
        container: 'border-orange-200 text-orange-800 dark:border-orange-800 dark:text-orange-300',
        gradient: 'from-orange-400 to-orange-500',
      };
    case 'idle':
    default:
      return {
        dot: 'bg-slate-700',
        ping: 'bg-slate-400',
        container: 'border-slate-200 text-slate-700 dark:border-slate-700 dark:text-slate-300',
        gradient: 'from-slate-400 to-slate-500',
      };
  }
};

const getSizeClasses = (size: StatusIndicatorProps['size']) => {
  switch (size) {
    case 'sm':
      return {
        dot: 'h-2 w-2',
        ping: 'h-2 w-2',
        container: 'px-2 py-1 text-xs',
      };
    case 'lg':
      return {
        dot: 'h-4 w-4',
        ping: 'h-4 w-4',
        container: 'px-4 py-2 text-sm',
      };
    case 'md':
    default:
      return {
        dot: 'h-3 w-3',
        ping: 'h-3 w-3',
        container: 'px-3 py-1.5 text-sm',
      };
  }
};

const StatusIndicator: React.FC<StatusIndicatorProps> = ({
  state = 'idle',
  label,
  className,
  size = 'md',
  labelClassName,
  animateAllStates = false,
}) => {
  const shouldAnimate = state === 'running' || state === 'pending' || animateAllStates;
  const colors = getStateColors(state);
  const sizeClasses = getSizeClasses(size);

  return (
    <div className='relative inline-flex rounded-full overflow-hiddenfrom-transparent via-transparent to-transparent'>
      <div
        className={cn(
          'relative inline-flex items-center gap-2 rounded-full bg-white dark:bg-slate-900 transition-all duration-300',
          sizeClasses.container,
          colors.container,
          className,
        )}
      >
        <div className='relative flex items-center'>
          {shouldAnimate && (
            <span
              className={cn(
                'absolute inline-flex rounded-full opacity-75 animate-ping',
                sizeClasses.ping,
                colors.ping,
              )}
            />
          )}
          <span className={cn('relative inline-flex rounded-full', sizeClasses.dot, colors.dot)} />
        </div>

        {label && <p className={cn('font-medium', labelClassName)}>{label}</p>}
      </div>
    </div>
  );
};

export default StatusIndicator;
