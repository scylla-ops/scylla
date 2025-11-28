import ReactCodeMirror, { EditorView } from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { toml } from '@codemirror/legacy-modes/mode/toml';
import { TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';

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
  return (
    <div className={'h-full'}>
      <TabsContent value='scripting' className={'h-full'}>
        <Card className={'h-full p-0'}>
          <ReactCodeMirror
            className='h-full'
            height='100%'
            extensions={[StreamLanguage.define(toml), codeMirrorTheme]}
          />
        </Card>
      </TabsContent>
      <TabsContent value='blueprint'>
        <p>Canvas reactflow ici</p>
      </TabsContent>
    </div>
  );
};
