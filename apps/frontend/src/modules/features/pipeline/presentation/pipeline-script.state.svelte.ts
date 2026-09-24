import type { PipelineStep } from '../domain/structs/pipeline.struct.ts';
import { nameOf, parseScript, stepsOf, withName, withSteps } from './utils/pipeline-script.ts';

export interface PipelineScriptParams {
  projectId: () => string;
  /** A default draft, or a fetched pipeline. */
  initialScript: () => string | undefined;
}

/** The editor's document: the script text is the single source of truth for both tabs. */
export const createPipelineScript = (params: PipelineScriptParams) => {
  let script = $state('');
  /** The saved state: the dirty check compares against it. */
  let baseline = $state('');

  // The document arrives after the first render; later edits must be kept.
  $effect(() => {
    const initial = params.initialScript();
    if (initial === undefined) return;

    baseline = initial;
    script = initial;
  });

  const parsed = $derived(parseScript(script));
  const steps = $derived(stepsOf(parsed.document));
  const pipelineName = $derived(nameOf(parsed.document));
  const isValid = $derived(parsed.document !== null && parsed.error === null);
  const isDirty = $derived(script !== baseline);

  // The browser's "leave site?" prompt, for a reload or a closed tab.
  $effect(() => {
    if (!isDirty) return;

    const warn = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      // Chrome still needs it to show the prompt.
      event.returnValue = '';
    };

    window.addEventListener('beforeunload', warn);
    return () => window.removeEventListener('beforeunload', warn);
  });

  return {
    get script() {
      return script;
    },
    get steps() {
      return steps;
    },
    get pipelineName() {
      return pipelineName;
    },
    get parseError() {
      return parsed.error;
    },
    get isValid() {
      return isValid;
    },
    get isDirty() {
      return isDirty;
    },

    setScript(next: string) {
      script = next;
    },

    setSteps(next: PipelineStep[]) {
      script = withSteps(parsed.document, next, params.projectId());
    },

    setName(name: string) {
      const renamed = withName(parsed.document, name);
      if (renamed !== null) script = renamed;
    },
  };
};

export type PipelineScript = ReturnType<typeof createPipelineScript>;
