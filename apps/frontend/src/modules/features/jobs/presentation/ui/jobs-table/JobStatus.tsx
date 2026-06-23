import { Badge } from '@shadcn';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { Trans } from '@lingui/react/macro';

type JobStatusProps = {
  job: JobEntity;
};

/**
 * Display the status badge for a job with icon and label
 */
export const JobStatus = ({ job }: JobStatusProps) => {
  const config = getStatusConfig(job.status);
  const Icon = config.icon;

  return (
    <div className='flex items-center gap-2'>
      <Icon className={`w-5 h-5 ${config.iconClassName}`} />
      <div className='flex flex-col'>
        <Badge variant={config.variant} className='w-fit'>
          {config.label}
        </Badge>
        {job.status === 'pending' ? (
          <span className='text-xs text-slate-500 truncate'>
            <Trans>queued — waiting for an agent</Trans>
          </span>
        ) : (
          <span className='text-xs text-slate-500 truncate'>{job.pipelineId}</span>
        )}
      </div>
    </div>
  );
};
