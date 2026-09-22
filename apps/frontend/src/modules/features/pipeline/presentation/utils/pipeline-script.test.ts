// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { nameOf, parseScript, stepsOf, withName, withSteps } from './pipeline-script';
import type { PipelineStep } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

const documentOf = (value: unknown) => parseScript(JSON.stringify(value)).document;

describe('parseScript', () => {
  it('an empty editor is not an error, just nothing to work with', () => {
    expect(parseScript('   ')).toEqual({ document: null, error: null });
  });

  it('reports malformed JSON without throwing', () => {
    const parsed = parseScript('{not json');
    expect(parsed.document).toBeNull();
    expect(parsed.error).not.toBeNull();
  });

  it('parses a well-formed document', () => {
    expect(parseScript('{"name":"my-ci"}').document).toEqual({ name: 'my-ci' });
  });
});

describe('nameOf', () => {
  it('falls back to "my-pipeline" for a document that names nothing', () => {
    expect(nameOf(null)).toBe('my-pipeline');
    expect(nameOf({})).toBe('my-pipeline');
  });

  it('reads the name the document carries', () => {
    expect(nameOf({ name: 'my-ci' })).toBe('my-ci');
  });
});

describe('stepsOf', () => {
  it('reads a fully specified step', () => {
    const steps = stepsOf(
      documentOf({ nodes: [{ id: 'build', deps: [], kind: 'exec', command: 'make', args: ['build'] }] }),
    );

    expect(steps).toEqual([
      { id: 'build', deps: [], workingDir: undefined, env: [], kind: 'exec', command: 'make', args: ['build'] },
    ]);
  });

  describe('node kind inference (no explicit "kind")', () => {
    it('a node with a "command" field is treated as exec', () => {
      const [step] = stepsOf(documentOf({ nodes: [{ id: 'a', command: 'echo hi' }] }));
      expect(step).toMatchObject({ kind: 'exec', command: 'echo hi' });
    });

    it('a node with a "script" field is treated as script', () => {
      const [step] = stepsOf(documentOf({ nodes: [{ id: 'a', script: 'echo hi' }] }));
      expect(step).toMatchObject({ kind: 'script', script: 'echo hi', shell: 'sh' });
    });

    it('a node with neither field defaults to an empty exec command', () => {
      const [step] = stepsOf(documentOf({ nodes: [{ id: 'a' }] }));
      expect(step).toMatchObject({ kind: 'exec', command: '', args: [] });
    });

    it('an explicit shell other than "bash" falls back to "sh"', () => {
      const [step] = stepsOf(
        documentOf({ nodes: [{ id: 'a', kind: 'script', script: 's', shell: 'zsh' }] }),
      );
      expect(step).toMatchObject({ shell: 'sh' });
    });
  });

  describe('env parsing', () => {
    it('tolerates a literal entry missing its value', () => {
      const [step] = stepsOf(documentOf({ nodes: [{ id: 'a', command: 'x', env: [{ key: 'K' }] }] }));
      expect(step.env).toEqual([{ key: 'K', kind: 'literal', value: '' }]);
    });

    it('tolerates a secret entry missing its secretRef', () => {
      const [step] = stepsOf(
        documentOf({ nodes: [{ id: 'a', command: 'x', env: [{ key: 'K', kind: 'secret' }] }] }),
      );
      expect(step.env).toEqual([{ key: 'K', kind: 'secret', secretRef: '' }]);
    });

    it('a non-array env is treated as empty rather than throwing', () => {
      const [step] = stepsOf(documentOf({ nodes: [{ id: 'a', command: 'x', env: 'nope' }] }));
      expect(step.env).toEqual([]);
    });
  });
});

describe('withSteps', () => {
  it('seeds a fresh {name, projectId} document when nothing was parsed yet', () => {
    const steps: PipelineStep[] = [
      { id: 'build', deps: [], env: [], kind: 'exec', command: 'make', args: [] },
    ];

    expect(JSON.parse(withSteps(null, steps, 'project-1'))).toEqual({
      name: 'my-pipeline',
      projectId: 'project-1',
      nodes: [
        { id: 'build', deps: [], workingDir: '', env: [], kind: 'exec', command: 'make', args: [] },
      ],
    });
  });

  it('preserves the rest of an already-parsed document — its name above all', () => {
    const steps: PipelineStep[] = [
      { id: 's', deps: [], env: [], kind: 'script', script: 'echo hi', shell: 'bash' },
    ];

    const parsed = JSON.parse(withSteps({ name: 'my-ci' }, steps, 'project-1')) as { name: string };
    expect(parsed.name).toBe('my-ci');
  });

  it('round-trips a step back through stepsOf, with an explicit empty workingDir', () => {
    const steps: PipelineStep[] = [
      { id: 's', deps: [], env: [], kind: 'script', script: 'echo hi', shell: 'bash' },
    ];

    // Serializing always writes a workingDir, unlike a hand-authored script
    // that omits the field entirely and parses back to `undefined`.
    expect(stepsOf(parseScript(withSteps(null, steps, 'p')).document)).toEqual([
      { id: 's', deps: [], workingDir: '', env: [], kind: 'script', script: 'echo hi', shell: 'bash' },
    ]);
  });
});

describe('withName', () => {
  it('is a no-op while nothing has been parsed', () => {
    expect(withName(null, 'new-name')).toBeNull();
  });

  it('renames the parsed document, keeping everything else', () => {
    const renamed = JSON.parse(withName({ name: 'old', projectId: 'p' }, 'new-name')!) as {
      name: string;
      projectId: string;
    };
    expect(renamed).toEqual({ name: 'new-name', projectId: 'p' });
  });
});
