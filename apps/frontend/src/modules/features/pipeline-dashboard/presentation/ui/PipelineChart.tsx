import { cn } from '@core/presentation/utils';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/modules/shared/presentation/ui/shadcn/tooltip';

const mockRuns = [1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 2];

export const PipelineChart = () => {
  return (
    <TooltipProvider delayDuration={100}>
      <div className='w-full flex items-center gap-2 h-10 py-1 overflow-hidden rounded-md   px-1'>
        {mockRuns.map((status, index) => (
          <Tooltip key={index}>
            <TooltipTrigger asChild>
              <div
                className={cn(
                  'flex-1 min-w-[4px] h-full rounded-sm transition-all duration-150 cursor-help',

                  // Success
                  status === 1 && 'bg-emerald-400/80 hover:bg-emerald-500 hover:scale-y-110',

                  // Error
                  status === 0 && 'bg-red-400/80 hover:bg-red-500 hover:scale-y-110',

                  // running
                  status === 2 &&
                    'bg-blue-500 animate-[smooth-pulse_2s_infinite] ring-4 ring-blue-400/30 ring-inset hover:scale-y-110',
                )}
              />
            </TooltipTrigger>
            <TooltipContent side='top' className='text-xs p-3 shadow-lg border-slate-200'>
              <div className='flex flex-col gap-1.5'>
                <div className='flex items-center justify-between gap-4'>
                  <span className='font-bold text-slate-400'>Run #{412 - index}</span>
                  <span className='text-[10px] text-slate-400 font-mono'>main • a7f2e1</span>
                </div>

                <div className='flex items-center gap-2'>
                  <div
                    className={cn(
                      'w-2 h-2 rounded-full',
                      status === 1 && 'bg-emerald-500',
                      status === 0 && 'bg-red-500',
                      status === 2 && 'bg-blue-500 animate-pulse',
                    )}
                  />
                  <span
                    className={cn(
                      'font-semibold',
                      status === 1 && 'text-emerald-600',
                      status === 0 && 'text-red-600',
                      status === 2 && 'text-blue-600',
                    )}
                  >
                    {status === 1 && 'Success'}
                    {status === 0 && 'Failed'}
                    {status === 2 && 'Running...'}
                  </span>
                </div>

                <span className='text-[10px] text-slate-500 italic border-t border-slate-100 pt-1 mt-1'>
                  Finished 2 mins ago • Duration: 1m 12s
                </span>
              </div>
            </TooltipContent>
          </Tooltip>
        ))}
      </div>
    </TooltipProvider>
  );
};
