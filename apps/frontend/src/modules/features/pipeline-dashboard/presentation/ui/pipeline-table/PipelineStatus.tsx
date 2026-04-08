import StatusIndicator from '@/modules/shared/presentation/ui/status-indicator';
import type { PipelineSummary } from '@/generated/pipeline';

type PipelineStatusProps = {
  pipeline: PipelineSummary;
};

/**
 * Component to display the status of a pipeline, including its name, branch, commit hash, and creation date.
 */
export const PipelineStatus = ({ pipeline }: PipelineStatusProps) => {
  const creationDate = new Date(pipeline.createdAt);

  return (
    <>
      <StatusIndicator state='running' />

      <div className='flex flex-col overflow-hidden'>
        <span className='font-semibold text-slate-900 truncate'>{pipeline.name}</span>
        <span className='text-xs font-mono text-slate-600 uppercase truncate'>main • a7f2e1</span>
        <span className='text-xs text-slate-500'>Creation: {creationDate.toDateString()}</span>
      </div>
    </>
  );
};
