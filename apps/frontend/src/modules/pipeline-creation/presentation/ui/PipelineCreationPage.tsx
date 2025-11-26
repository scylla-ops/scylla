import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { toml } from '@codemirror/legacy-modes/mode/toml';

export const PipelineCreationPage = () => {
  return (
    <ReactCodeMirror
      className={'h-full'}
      height={'100%'}
      extensions={[StreamLanguage.define(toml)]}
    />
  );
};
