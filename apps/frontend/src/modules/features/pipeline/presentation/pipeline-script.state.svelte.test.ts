import { describe, it, expect, vi, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { createPipelineScript } from './pipeline-script.state.svelte.ts';

const withScript = (
  initialScript: string | undefined,
  run: (script: ReturnType<typeof createPipelineScript>) => void,
) => {
  const cleanup = $effect.root(() => {
    const script = createPipelineScript({
      projectId: () => 'project-1',
      initialScript: () => initialScript,
    });
    flushSync();
    run(script);
  });
  cleanup();
};

const document = JSON.stringify({
  name: 'nightly',
  nodes: [{ id: 'build', deps: [], kind: 'exec', command: 'make', args: [] }],
});

afterEach(() => vi.restoreAllMocks());

describe('createPipelineScript', () => {
  it('starts from the initial script, clean', () => {
    withScript(document, script => {
      expect(script.script).toBe(document);
      expect(script.pipelineName).toBe('nightly');
      expect(script.steps.map(step => step.id)).toEqual(['build']);
      expect(script.isValid).toBe(true);
      expect(script.isDirty).toBe(false);
    });
  });

  it('waits while the initial script is not loaded', () => {
    withScript(undefined, script => {
      expect(script.script).toBe('');
      expect(script.isValid).toBe(false);
    });
  });

  it('becomes dirty and reports a parse error for an invalid edit', () => {
    withScript(document, script => {
      script.setScript('{not json');
      flushSync();

      expect(script.isDirty).toBe(true);
      expect(script.isValid).toBe(false);
      expect(script.parseError).not.toBeNull();
    });
  });

  it('writes new steps and a new name into the document', () => {
    withScript(document, script => {
      script.setSteps([
        { id: 'test', deps: [], kind: 'script', script: 'make test', shell: 'sh', env: [] },
      ]);
      script.setName('weekly');
      flushSync();

      expect(script.pipelineName).toBe('weekly');
      expect(script.steps.map(step => step.id)).toEqual(['test']);
    });
  });

  it('warns before the page unloads while there are unsaved changes', () => {
    const addEventListener = vi.spyOn(window, 'addEventListener');
    withScript(document, script => {
      script.setScript(`${document} `);
      flushSync();

      expect(addEventListener).toHaveBeenCalledWith('beforeunload', expect.any(Function));
    });
  });
});
