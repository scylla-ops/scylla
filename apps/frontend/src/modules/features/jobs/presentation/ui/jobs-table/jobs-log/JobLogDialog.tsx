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
import { CopyableText } from '@shared/presentation/ui/data-display/CopyableText.tsx';

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
      <DialogContent
        className={'max-w-5xl max-h-full flex flex-col'}
        // avoids autofocus landing on a copy button, which swallows Escape
        onOpenAutoFocus={e => e.preventDefault()}
      >
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
                <Badge
                  variant={nodeId ? 'outline' : 'default'}
                  className={'font-mono text-xs gap-1 pr-1'}
                >
                  <CopyableText
                    value={jobId ?? ''}
                    display={<>#{jobId}</>}
                    copyLabel={<Trans>Copy job id</Trans>}
                    copyButtonClassName='h-5 w-5 hover:scale-100'
                  />
                </Badge>
                {nodeId && (
                  <Badge variant={'default'} className={'font-mono text-xs gap-1 pr-1'}>
                    <CopyableText
                      value={nodeId}
                      display={
                        <>
                          <Trans>Node</Trans> #{nodeId}
                        </>
                      }
                      copyLabel={<Trans>Copy node id</Trans>}
                      copyButtonClassName='h-5 w-5 hover:scale-100'
                    />
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
