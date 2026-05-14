import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { shell } from '@codemirror/legacy-modes/mode/shell';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { useTailJobLogs } from '@/modules/features/jobs/presentation/hooks/use-tail-job-logs.ts';

interface JobLogDisplayProps {
  jobId: string;
  nodeId?: string;
}

export const JobLogDisplay = ({ jobId, nodeId }: JobLogDisplayProps) => {
  const { logString, isError, isLoading } = useTailJobLogs(jobId, nodeId);

  if (isLoading) return <div>Loading...</div>;
  if (isError) return <div>Error loading logs...</div>;

  return (
    <div className={'border rounded-md overflow-hidden shadow-sm'}>
      <ReactCodeMirror
        readOnly
        editable={false}
        autoFocus={false}
        value={logString}
        maxHeight={'15rem'}
        extensions={[StreamLanguage.define(shell), codeMirrorTheme]}
      />
    </div>
  );
};
