import React from 'react';
import { cn } from '@core/presentation/utils';

interface StatusIndicatorProps {
  state: 'success' | 'failure' | 'running' | 'idle';
  label?: string;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  labelClassName?: string;
}

//TODO: add spinning border when state is running

const getStateColors = (state: StatusIndicatorProps['state']) => {
  switch (state) {
    case 'success':
      return {
        dot: 'bg-green-500',
        ping: 'bg-green-300',
        container:
          'bg-green-50 border-green-200 text-green-800 dark:bg-green-900/20 dark:border-green-800 dark:text-green-300',
      };
    case 'failure':
      return {
        dot: 'bg-red-500',
        ping: 'bg-red-300',
        container:
          'bg-red-50 border-red-200 text-red-800 dark:bg-red-900/20 dark:border-red-800 dark:text-red-300',
      };
    case 'running':
      return {
        dot: 'bg-blue-500',
        ping: 'bg-blue-300',
        container:
          'bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-900/20 dark:border-blue-800 dark:text-blue-300',
      };
    case 'idle':
    default:
      return {
        dot: 'bg-slate-700',
        ping: 'bg-slate-400',
        container:
          'bg-slate-100 border-slate-200 text-slate-700 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300',
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
}) => {
  const shouldAnimate = state === 'success' || state === 'running' || state === 'failure';

  const colors = getStateColors(state);
  const sizeClasses = getSizeClasses(size);

  return (
    <>
      <div
        className={cn(
          'inline-flex items-center gap-2 rounded-full border px-3 py-1.5',
          'transition-colors',
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

        {label && <p className={cn('text-sm font-medium', labelClassName)}>{label}</p>}
      </div>
    </>
  );
};

export default StatusIndicator;
