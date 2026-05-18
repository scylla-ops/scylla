import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { shell } from '@codemirror/legacy-modes/mode/shell';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { useTailJobLogs } from '@/modules/features/jobs/presentation/hooks/use-tail-job-logs.ts';
import { useJobLogs } from '@/modules/features/jobs/presentation/hooks/use-job-logs.ts';
import { useMemo } from 'react';

interface JobLogDisplayProps {
  jobId: string;
  nodeId?: string;
}

interface LogViewerProps {
  logs: string;
  isLoading: boolean;
  isError: boolean;
}

const LogViewer = ({ logs, isLoading, isError }: LogViewerProps) => {
  if (isLoading) return <div>Loading...</div>;
  if (isError) return <div>Error loading logs...</div>;

  return (
    <div className={'border rounded-md overflow-hidden shadow-sm'}>
      <ReactCodeMirror
        readOnly
        editable={false}
        autoFocus={false}
        value={logs}
        maxHeight={'15rem'}
        extensions={[StreamLanguage.define(shell), codeMirrorTheme]}
      />
    </div>
  );
};

const StreamingJobLogs = ({ jobId }: { jobId: string }) => {
  const { logString, isLoading, isError } = useTailJobLogs(jobId);

  return <LogViewer logs={logString} isLoading={isLoading} isError={isError} />;
};

const FetchedJobLogs = ({ jobId, nodeId }: { jobId: string; nodeId: string }) => {
  const { logs, isLoading, isError } = useJobLogs(jobId, nodeId);

  const formattedLogs = useMemo(
    () => logs?.items.map(log => log.line).join('\n') ?? '',
    [logs],
  );

  return <LogViewer logs={formattedLogs} isLoading={isLoading} isError={isError} />;
};

export const JobLogDisplay = ({ jobId, nodeId }: JobLogDisplayProps) => {
  if (nodeId) {
    return <FetchedJobLogs jobId={jobId} nodeId={nodeId} />;
  }

  return <StreamingJobLogs jobId={jobId} />;
};
