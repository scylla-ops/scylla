import StatusIndicator from '@shared/presentation/ui/status-indicator.tsx';
import type { PipelineSummary } from '@/generated/pipeline.ts';
import { Trans } from '@lingui/react/macro';

type PipelineStatusProps = {
  pipeline: PipelineSummary;
  status?: 'idle' | 'running' | 'success' | 'failed';
};

/**
 * Component to display the status of a pipeline, including its name, branch, commit hash, and creation date.
 */
export const PipelineStatus = ({ pipeline, status }: PipelineStatusProps) => {
  const creationDate = new Date(pipeline.createdAt);

  return (
    <div className={'flex items-center justify-center gap-2 flex-row'}>
      <StatusIndicator state={status || 'idle'} />

      <div className='flex flex-col overflow-hidden'>
        <span className='font-semibold text-slate-900 truncate'>{pipeline.name}</span>
        <span className='text-xs text-slate-500'>
          <Trans>Creation:</Trans> {creationDate.toDateString()}
        </span>
      </div>
    </div>
  );
};
