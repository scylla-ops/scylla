import ReactCodeMirror, { type ReactCodeMirrorRef } from '@uiw/react-codemirror';
import { useCodeMirrorTheme } from '@shared/presentation/hooks/use-code-mirror-theme.ts';
import { useTailJobLogs } from '@/modules/features/jobs/presentation/hooks/use-tail-job-logs.ts';
import { Trans } from '@lingui/react/macro';
import { useEffect, useRef } from 'react';

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

  const editorRef = useRef<ReactCodeMirrorRef>(null);

  useEffect(() => {
    const view = editorRef.current?.view;

    if (view) {
      requestAnimationFrame(() => {
        const scrollEl = view.scrollDOM;

        scrollEl.scrollTo({
          top: scrollEl.scrollHeight,
          behavior: 'smooth',
        });
      });
    }
  }, [logs]);

  if (isLoading)
    return (
      <div>
        <Trans>Loading...</Trans>
      </div>
    );
  if (isError)
    return (
      <div>
        <Trans>Error loading logs...</Trans>
      </div>
    );

  return (
    <div className={'min-w-0 w-full rounded-xl overflow-hidden shadow-sm'}>
      <ReactCodeMirror
        readOnly
        editable={false}
        autoFocus={false}
        value={logs}
        maxHeight={'28rem'}
        theme={editorTheme}
        ref={editorRef}
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
