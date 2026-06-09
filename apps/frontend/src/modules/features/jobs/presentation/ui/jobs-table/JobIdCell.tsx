import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
import { useState } from 'react';
import { Button } from '@shadcn';
import { Check, Copy } from 'lucide-react';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans } from '@lingui/react/macro';

export function JobIdCell({ job }: { job: Job }) {
  const [copied, setCopied] = useState(false);

  const handleCopyJobId = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard.writeText(job.id);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className='flex items-center gap-2'>
      <span className='font-mono text-sm truncate'>{job.id.slice(0, 12)}...</span>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button size='icon' variant='ghost' className='h-6 w-6 shrink-0 cursor-pointer transition-all hover:scale-125 hover:text-primary hover:bg-primary-hover rounded-full' onClick={handleCopyJobId}>
            {copied ? <Check className='w-3 h-3 text-green-500' /> : <Copy className='w-3 h-3' />}
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>{copied ? <Trans>Copied!</Trans> : <Trans>Copy ID</Trans>}</p>
        </TooltipContent>
      </Tooltip>
    </div>
  );
}

export default JobIdCell;
