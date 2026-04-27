import ReactCodeMirror, { EditorView } from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { PipelineCreationTopbar } from '@/modules/features/pipeline/presentation/ui/creation/PipelineCreationTopbar.tsx';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';

const codeMirrorTheme = EditorView.theme({
  '&': {
    borderRadius: '0.75rem',
    overflow: 'hidden',
  },
  '.cm-scroller': {
    overflow: 'auto',
  },
  '.cm-content': {
    padding: '0.5rem',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--code-editor-line-bg)',
  },
  '.cm-activeLine': {
    backgroundColor: 'var(--code-editor-line-bg)',
  },
  'cm-gutters': {
    backgroundColor: 'transparent',
    border: 'none',
  },
  '&.cm-focused': {
    outline: 'none',
    boxShadow: `
    -2px 0 0px 0px var(--primary),
    0 1px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06)
  `,
    borderRadius: '0.75rem',
    transition: 'box-shadow 0.1s ease-in-out',
  },
});

export const PipelineCreationPage = () => {
  const { script, setScript } = useScriptStore(state => state);

  const { projectId } = useParams();

  //TODO: change that
  useEffect(() => {
    if (projectId) {
      setScript(
        `{\n"name": "my-pipeline",\n` +
          `"projectId": "${projectId}",\n` +
          `"nodes": [
    {
      "nodeId": "build",
      "deps": [],
      "command": "cargo",
      "args": ["build", "--release"]
    },
    {
      "nodeId": "test",
      "deps": ["build"],
      "command": "cargo",
      "args": ["test"]
    }
  ]\n` +
          `}`,
      );
    }
  }, [projectId, setScript]);

  if (!projectId)
    return (
      <p>
        <Trans>Select a project first</Trans>
      </p>
    );

  return (
    <Tabs key={'scripting'} defaultValue={'scripting'} className={'h-full flex flex-col gap-4'}>
      <PipelineCreationTopbar />
      <TabsContent value='scripting' className={'h-full'}>
        <Card className={'h-full p-0'}>
          <ReactCodeMirror
            value={script}
            onChange={value => setScript(value)}
            className='h-full'
            height='100%'
            extensions={[StreamLanguage.define(json), codeMirrorTheme]}
          />
        </Card>
      </TabsContent>
      <TabsContent value='blueprint'>
        <p>
          <Trans>Reactflow canvas here</Trans>
        </p>
      </TabsContent>
    </Tabs>
  );
};
