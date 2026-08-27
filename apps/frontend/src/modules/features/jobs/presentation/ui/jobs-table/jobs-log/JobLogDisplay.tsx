import ReactCodeMirror from '@uiw/react-codemirror';
import { useCodeMirrorTheme } from '@shared/presentation/hooks/use-code-mirror-theme.ts';
import { useTailJobLogs } from '@/modules/features/jobs/presentation/hooks/use-tail-job-logs.ts';

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
  const editorTheme = useCodeMirrorTheme();

  if (isLoading) return <div>Loading...</div>;
  if (isError) return <div>Error loading logs...</div>;

  return (
    <div className={'min-w-0 w-full rounded-xl overflow-hidden shadow-sm'}>
      <ReactCodeMirror
        readOnly
        editable={false}
        autoFocus={false}
        value={logs}
        maxHeight={'28rem'}
        theme={editorTheme}
      />
    </div>
  );
};

/**
 * Job log view. Both the whole-job and per-node views use the same streaming
 * source (full persisted history + live tail), so logs are complete and live
 * regardless of when the view is opened.
 */
export const JobLogDisplay = ({ jobId, nodeId }: JobLogDisplayProps) => {
  const { logString, isLoading, isError } = useTailJobLogs(jobId, nodeId);

  return <LogViewer logs={logString} isLoading={isLoading} isError={isError} />;
};
