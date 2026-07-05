import {
  Badge,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@shadcn';
import { Trans } from '@lingui/react/macro';
import { Radio, Terminal } from 'lucide-react';

import { JobLogDisplay } from '@/modules/features/jobs/presentation/ui/jobs-table/jobs-log/JobLogDisplay.tsx';

interface JobLogDialogProps {
  jobId?: string;
  nodeId?: string;
  onClose: () => void;
}

export const JobLogDialog = ({ jobId, nodeId, onClose }: JobLogDialogProps) => {
  return (
    <Dialog
      open={jobId !== undefined}
      onOpenChange={open => {
        if (!open) {
          onClose();
        }
      }}
    >
      <DialogContent className={'max-w-3xl max-h-full flex flex-col'}>
        <DialogHeader className={'space-y-3'}>
          <div className={'flex items-center justify-between'}>
            <DialogTitle className={'flex items-center gap-2.5 text-lg font-semibold'}>
              <div
                className={'flex items-center justify-center size-8 rounded-lg bg-primary/10 mr-2'}
              >
                <Terminal className={'size-4 text-primary'} />
              </div>
              <div className={'flex items-center gap-2'}>
                <span>
                  <Trans>Job Logs</Trans>
                </span>
                <Badge variant={nodeId ? 'outline' : 'default'} className={'font-mono text-xs'}>
                  #{jobId}
                </Badge>
                {nodeId && (
                  <Badge variant={'default'} className={'font-mono text-xs'}>
                    Node #{nodeId}
                  </Badge>
                )}
              </div>
            </DialogTitle>
          </div>
          <DialogDescription className={'flex items-center gap-1.5 text-muted-foreground text-sm'}>
            <Radio className={'size-4 animate-pulse text-green-500'} />
            <Trans>Streaming live output</Trans>
          </DialogDescription>
        </DialogHeader>
        {jobId && <JobLogDisplay jobId={jobId} nodeId={nodeId} />}
      </DialogContent>
    </Dialog>
  );
};
