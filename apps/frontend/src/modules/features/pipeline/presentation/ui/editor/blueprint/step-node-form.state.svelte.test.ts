import { describe, it, expect } from 'vitest';
import { createStepNodeForm } from './step-node-form.state.svelte.ts';

const inRoot = (run: () => void) => {
  const cleanup = $effect.root(run);
  cleanup();
};

describe('createStepNodeForm', () => {
  it('starts empty, in script mode, with sh', () => {
    inRoot(() => {
      const form = createStepNodeForm();
      expect(form.nodeId).toBe('');
      expect(form.mode).toBe('script');
      expect(form.shell).toBe('sh');
      expect(form.toValue()).toBeNull();
    });
  });

  it('starts from the step it edits', () => {
    inRoot(() => {
      const form = createStepNodeForm({
        id: 'build',
        deps: [],
        kind: 'exec',
        command: 'cargo',
        args: ['build'],
        workingDir: 'crates',
        env: [
          { key: 'A', kind: 'literal', value: '1' },
          { key: 'B', kind: 'secret', secretRef: 'TOKEN' },
        ],
      });
      expect(form.mode).toBe('exec');
      expect(form.command).toBe('cargo');
      expect(form.args).toEqual(['build']);
      expect(form.envRows).toEqual([
        { key: 'A', kind: 'literal', value: '1', secretRef: '' },
        { key: 'B', kind: 'secret', value: '', secretRef: 'TOKEN' },
      ]);
    });
  });

  it('gives a script step, trimmed, without blank environment rows', () => {
    inRoot(() => {
      const form = createStepNodeForm();
      form.nodeId = '  build  ';
      form.script = 'make';
      form.shell = 'bash';
      form.workingDir = '  ';
      form.addEnv();
      form.addEnv();
      form.setEnv(0, { key: ' A ', value: '1' });

      expect(form.toValue()).toEqual({
        nodeId: 'build',
        value: {
          kind: 'script',
          script: 'make',
          shell: 'bash',
          workingDir: undefined,
          env: [{ key: 'A', kind: 'literal', value: '1' }],
        },
      });
    });
  });

  it('refuses a script step with an empty script', () => {
    inRoot(() => {
      const form = createStepNodeForm();
      form.nodeId = 'build';
      form.script = '   ';
      expect(form.toValue()).toBeNull();
    });
  });

  it('gives an exec step without the blank arguments', () => {
    inRoot(() => {
      const form = createStepNodeForm();
      form.nodeId = 'build';
      form.mode = 'exec';
      form.command = ' cargo ';
      form.addArg();
      form.addArg();
      form.addArg();
      form.setArg(0, 'build');
      form.setArg(2, '--release');
      form.removeArg(1);
      form.workingDir = ' crates ';
      form.addEnv();
      form.setEnv(0, { key: 'TOKEN', kind: 'secret', secretRef: 'GH' });

      expect(form.toValue()).toEqual({
        nodeId: 'build',
        value: {
          kind: 'exec',
          command: 'cargo',
          args: ['build', '--release'],
          workingDir: 'crates',
          env: [{ key: 'TOKEN', kind: 'secret', secretRef: 'GH' }],
        },
      });
    });
  });

  it('refuses an exec step without a command', () => {
    inRoot(() => {
      const form = createStepNodeForm();
      form.nodeId = 'build';
      form.mode = 'exec';
      expect(form.toValue()).toBeNull();
    });
  });

  it('removes an environment row', () => {
    inRoot(() => {
      const form = createStepNodeForm();
      form.addEnv();
      form.addEnv();
      form.setEnv(1, { key: 'KEEP' });
      form.removeEnv(0);
      expect(form.envRows.map(row => row.key)).toEqual(['KEEP']);
    });
  });
});
