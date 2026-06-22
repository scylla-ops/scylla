import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useEffect } from 'react';
import { Trans } from '@lingui/react/macro';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { PipelineEditorHeader } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditorHeader.tsx';
import {
  codeMirrorTheme,
  codeMirrorErrorTheme,
} from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { PipelineBlueprint } from '@/modules/features/pipeline/presentation/ui/editor/blueprint/PipelineBlueprint.tsx';
import { usePipelineScript } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-script.ts';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

interface PipelineEditorProps {
  /** `create` shows a neutral draft state; `edit` tracks dirty/saved divergence. */
  mode: 'create' | 'edit';
  /** Label for the submit button (e.g. "Create" / "Save"). */
  submitLabel: string;
  /** Called with the parsed pipeline when the user submits a valid script. */
  onSubmit: (values: { name: string; steps: PipelineStep[] }) => void;

  /** Initial script to load into the editor once available (default or fetched). */
  initialScript?: string;
  /** Project id, used as fallback when serializing blueprint edits. */
  projectId?: string;

  isSubmitPending?: boolean;
}

export const PipelineEditor = ({
  mode,
  submitLabel,
  onSubmit,
  initialScript,
  projectId,
  isSubmitPending = false,
}: PipelineEditorProps) => {
  const {
    script,
    setScript,
    pipelineName,
    steps,
    isValid,
    parseError,
    setInitialScript,
    isDirty,
    handleStepsChange,
    handleNameChange,
  } = usePipelineScript({ projectId });

  useEffect(() => {
    if (initialScript) {
      setInitialScript(initialScript);
      setScript(initialScript);
    }
  }, [initialScript, setInitialScript, setScript]);

  const handleSubmit = () => {
    if (!isValid) return;
    onSubmit({ name: pipelineName, steps });
  };

  //todo: confirmation also if user navigate using breadcrumb (need refactor of the breadcrumb system (using a store?) )
  useEffect(() => {
    if (!isDirty) return;

    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = '';
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [isDirty]);

  return (
    <Tabs key='editor' defaultValue='blueprint' className='flex h-full flex-col gap-4'>
      <div className='flex w-full items-center justify-between gap-4'>
        <PipelineEditorHeader
          onSubmit={handleSubmit}
          submitLabel={submitLabel}
          mode={mode}
          submitDisabled={!isValid}
          blueprintDisabled={!isValid}
          isDirty={isDirty}
          isSaving={isSubmitPending}
        />
      </div>
      <TabsContent value='scripting' className='h-full overflow-hidden' forceMount>
        <div className='flex h-full flex-col gap-2'>
          <div className={'overflow-auto p-2'}>
            <Card className='min-h-0 flex-1 p-0'>
              <ReactCodeMirror
                value={script}
                onChange={setScript}
                className='h-full'
                height='100%'
                extensions={[
                  StreamLanguage.define(json),
                  parseError ? codeMirrorErrorTheme : codeMirrorTheme,
                ]}
              />
            </Card>
          </div>
          {parseError && (
            <p className='text-destructive text-sm'>
              <Trans>Invalid JSON</Trans>: {parseError}
            </p>
          )}
        </div>
      </TabsContent>
      <TabsContent value='blueprint' className='h-full'>
        <Card className='h-full p-0'>
          <PipelineBlueprint
            steps={steps}
            pipelineName={pipelineName}
            onStepsChange={handleStepsChange}
            onNameChange={handleNameChange}
          />
        </Card>
      </TabsContent>
    </Tabs>
  );
};
