import StatusIndicator, {
  type StatusState,
} from '@shared/presentation/ui/data-display/status-indicator.tsx';
import { Trans } from '@lingui/react/macro';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

type PipelineStatusProps = {
  pipeline: PipelineMetadata;
  status: StatusState;
};

/**
 * Component to display the status of a pipeline, including its name, branch, commit hash, and creation date.
 */
export const PipelineStatus = ({ pipeline, status }: PipelineStatusProps) => {
  const creationDate = new Date(pipeline.createdAt);

  return (
    <div className={'flex items-center justify-center gap-2 flex-row'}>
      <StatusIndicator state={status} />

      <div className='flex flex-col overflow-hidden'>
        <span className='font-semibold text-slate-900 truncate'>{pipeline.name}</span>
        <span className='text-xs text-slate-500'>
          <Trans>Creation:</Trans> {creationDate.toDateString()}
        </span>
        <span className={'text-xs text-slate-400'}>id: {pipeline.id}</span>
      </div>
    </div>
  );
};
