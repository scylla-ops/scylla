import StatusIndicator, {
  type StatusState,
} from '@shared/presentation/ui/data-display/status-indicator.tsx';
import { Trans } from '@lingui/react/macro';
import { formatDay } from '@shared/utils/date-utils.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import { CopyableText } from '@shared/presentation/ui';

type PipelineStatusProps = {
  pipeline: PipelineMetadata;
  status: StatusState;
};

/**
 * Component to display the status of a pipeline, including its name, branch, commit hash, and creation date.
 */
export const PipelineStatus = ({ pipeline, status }: PipelineStatusProps) => {
  return (
    <div className='flex w-full flex-row items-center justify-start gap-2'>
      {/* The pill is not the line that should give way when the column narrows. */}
      <div className='shrink-0'>
        <StatusIndicator state={status} />
      </div>

      {/* Stretched, not `items-start`: on the cross axis a flex-start child is sized to
          its own text, so `truncate`'s nowrap made every line as wide as its content and
          there was nothing left to ellipsize — it just overflowed into the clipped cell. */}
      <div className='flex min-w-0 flex-1 flex-col text-start'>
        <span className='truncate font-semibold text-foreground'>{pipeline.name}</span>
        <span className='truncate text-xs text-muted-foreground'>
          <Trans>Creation:</Trans> {formatDay(pipeline.createdAt)}
        </span>
        <CopyableText className='text-xs text-muted-foreground/80' value={pipeline.id} />
      </div>
    </div>
  );
};
