import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { shell } from '@codemirror/legacy-modes/mode/shell';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
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
  if (isLoading) return <div>Loading...</div>;
  if (isError) return <div>Error loading logs...</div>;

  return (
    <div className={'min-w-0 w-full border rounded-md overflow-hidden shadow-sm'}>
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

/**
 * Job log view. Both the whole-job and per-node views use the same streaming
 * source (full persisted history + live tail), so logs are complete and live
 * regardless of when the view is opened.
 */
export const JobLogDisplay = ({ jobId, nodeId }: JobLogDisplayProps) => {
  const { logString, isLoading, isError } = useTailJobLogs(jobId, nodeId);

  return <LogViewer logs={logString} isLoading={isLoading} isError={isError} />;
};
