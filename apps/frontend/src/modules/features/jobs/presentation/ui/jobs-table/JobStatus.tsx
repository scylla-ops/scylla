import { Badge } from '@shadcn';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
import { getStatusConfig } from '@shared/utils/status-config.ts';

type JobStatusProps = {
  job: Job;
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
        <span className='text-xs text-slate-500 truncate'>{job.pipelineId}</span>
      </div>
    </div>
  );
};
