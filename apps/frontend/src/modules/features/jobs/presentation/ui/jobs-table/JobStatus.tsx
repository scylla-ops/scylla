import { Badge } from '@shadcn';
import { CheckCircle2, XCircle, Clock, Loader2 } from 'lucide-react';
import type { JobResponse } from '@/generated/job.ts';

type JobStatusProps = {
  job: JobResponse;
};

const statusConfig: Record<
  string,
  {
    label: string;
    variant: 'default' | 'secondary' | 'destructive' | 'outline';
    icon: typeof CheckCircle2;
    className?: string;
  }
> = {
  pending: {
    label: 'Pending',
    variant: 'secondary',
    icon: Clock,
    className: 'text-slate-500',
  },
  running: {
    label: 'Running',
    variant: 'default',
    icon: Loader2,
    className: 'text-blue-500 animate-spin',
  },
  success: {
    label: 'Success',
    variant: 'outline',
    icon: CheckCircle2,
    className: 'text-green-500',
  },
  failed: {
    label: 'Failed',
    variant: 'destructive',
    icon: XCircle,
    className: 'text-red-500',
  },
};

/**
 * Display the status badge for a job with icon and label
 */
export const JobStatus = ({ job }: JobStatusProps) => {
  const config = statusConfig[job.status] || statusConfig.pending;
  const Icon = config.icon;

  return (
    <div className='flex items-center gap-2'>
      <Icon className={`w-5 h-5 ${config.className}`} />
      <div className='flex flex-col'>
        <Badge variant={config.variant} className='w-fit'>
          {config.label}
        </Badge>
        <span className='text-xs text-slate-500 truncate'>{job.pipelineId}</span>
      </div>
    </div>
  );
};
