import type { JobResponse } from '@/generated/job.ts';
import { useState } from 'react';
import { Button } from '@shadcn';
import { Check, Copy } from 'lucide-react';

export function JobIdCell({ job }: { job: JobResponse }) {
  const [copied, setCopied] = useState(false);

  const handleCopyJobId = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(job.jobId);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className='flex items-center gap-2'>
      <span className='font-mono text-sm truncate'>{job.jobId.slice(0, 12)}...</span>
      <Button size='icon' variant='ghost' className='h-6 w-6 shrink-0' onClick={handleCopyJobId}>
        {copied ? <Check className='w-3 h-3 text-green-500' /> : <Copy className='w-3 h-3' />}
      </Button>
    </div>
  );
}

export default JobIdCell;
