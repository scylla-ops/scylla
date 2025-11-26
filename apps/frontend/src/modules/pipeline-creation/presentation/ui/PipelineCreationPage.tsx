import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { toml } from '@codemirror/legacy-modes/mode/toml';

export const PipelineCreationPage = () => {
  return <ReactCodeMirror extensions={[StreamLanguage.define(toml)]}></ReactCodeMirror>;
};
