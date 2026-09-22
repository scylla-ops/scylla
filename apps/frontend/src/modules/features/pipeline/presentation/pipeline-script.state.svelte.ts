import type { PipelineStep } from '../domain/structs/pipeline.struct.ts';
import { nameOf, parseScript, stepsOf, withName, withSteps } from './utils/pipeline-script.ts';

export interface PipelineScriptParams {
  /** Fallback project id, written into a document seeded from an empty editor. */
  projectId: () => string;
  /** The document to open with — a default draft, or a fetched pipeline. */
  initialScript: () => string | undefined;
}

/**
 * The editor's document: one string, and everything both tabs read off it.
 *
 * The script text is the single source of truth. The blueprint does not hold a
 * graph of its own that has to be reconciled — it reads `steps` and writes back
 * through `setSteps`, so the two tabs cannot disagree about what the pipeline
 * is. That was already the design; what the port removes is the Zustand store
 * it lived in. It never needed to be global: one editor is open at a time, and
 * the store's only job was to let a hook and a component share it.
 *
 * `parsed` is computed once and everything else derives from it, so a malformed
 * script produces exactly one error and no half-parsed state.
 */
export const createPipelineScript = (params: PipelineScriptParams) => {
  let script = $state('');
  /** What "saved" looks like — the dirty check is a comparison against it. */
  let baseline = $state('');

  // The document arrives from outside and later than this state: a fetched
  // pipeline resolves after the page renders. Seeding it is synchronisation,
  // not derivation — the user edits `script` afterwards and must keep those
  // edits.
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

  // The browser's own "leave site?" prompt is the only thing that can catch a
  // reload or a closed tab, and it is a window-level subscription — a system
  // outside the app, which is what `$effect` is for.
  $effect(() => {
    if (!isDirty) return;

    const warn = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      // Still required by Chrome to raise the dialog, deprecated or not.
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

    /** The blueprint's edits, written back into the document they came from. */
    setSteps(next: PipelineStep[]) {
      script = withSteps(parsed.document, next, params.projectId());
    },

    /** A rename is a no-op while there is no document to rename. */
    setName(name: string) {
      const renamed = withName(parsed.document, name);
      if (renamed !== null) script = renamed;
    },
  };
};

export type PipelineScript = ReturnType<typeof createPipelineScript>;
