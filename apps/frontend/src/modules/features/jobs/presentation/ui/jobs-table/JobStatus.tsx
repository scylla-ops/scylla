import { Badge } from '@shadcn';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { TruncatedText } from '@shared/presentation/ui';
import { Trans } from '@lingui/react/macro';
import { useLingui } from '@lingui/react';

type JobStatusProps = {
  job: JobEntity;
};

/**
 * Display the status badge for a job with icon and label
 */
export const JobStatus = ({ job }: JobStatusProps) => {
  const { _ } = useLingui();
  const config = getStatusConfig(job.status);
  const Icon = config.icon;

  return (
    <div className='w-full flex items-center gap-2'>
      <Icon className={`w-5 h-5 ${config.iconClassName}`} />
      <div className='flex min-w-0 flex-col'>
        <Badge variant={config.variant} className='w-fit'>
          {_(config.label)}
        </Badge>
        {job.status === 'pending' && (
          <TruncatedText className='text-xs text-muted-foreground'>
            <Trans>queued — waiting for an agent</Trans>
          </TruncatedText>
        )}
        {job.status === 'orphaned' && (
          <TruncatedText className='text-xs text-muted-foreground'>
            <Trans>agent disconnected mid-run</Trans>
          </TruncatedText>
        )}
      </div>
    </div>
  );
};
