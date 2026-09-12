import { describe, it, expect, beforeEach } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { usePipelineScript } from './use-pipeline-script';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';

beforeEach(() => {
  useScriptStore.setState({ script: '', initialScript: '' });
});

describe('usePipelineScript', () => {
  it('an empty editor is not an error, just not yet saveable', () => {
    const { result } = renderHook(() => usePipelineScript());

    expect(result.current.parseError).toBeNull();
    expect(result.current.isValid).toBe(false);
    expect(result.current.pipelineName).toBe('my-pipeline');
    expect(result.current.steps).toEqual([]);
    expect(result.current.isDirty).toBe(false);
  });

  it('surfaces a parse error for malformed JSON, and stays invalid', () => {
    useScriptStore.setState({ script: '{not json', initialScript: '' });
    const { result } = renderHook(() => usePipelineScript());

    expect(result.current.parseError).not.toBeNull();
    expect(result.current.isValid).toBe(false);
    expect(result.current.steps).toEqual([]);
  });

  it('parses a valid script into its pipeline name and steps', () => {
    useScriptStore.setState({
      script: JSON.stringify({
        name: 'my-ci',
        nodes: [{ id: 'build', deps: [], kind: 'exec', command: 'make', args: ['build'] }],
      }),
    });
    const { result } = renderHook(() => usePipelineScript());

    expect(result.current.isValid).toBe(true);
    expect(result.current.pipelineName).toBe('my-ci');
    expect(result.current.steps).toEqual([
      { id: 'build', deps: [], workingDir: undefined, env: [], kind: 'exec', command: 'make', args: ['build'] },
    ]);
  });

  describe('node kind inference (no explicit "kind")', () => {
    it('a node with a "command" field is treated as exec', () => {
      useScriptStore.setState({
        script: JSON.stringify({ nodes: [{ id: 'a', command: 'echo hi' }] }),
      });
      const { result } = renderHook(() => usePipelineScript());
      expect(result.current.steps[0]).toMatchObject({ kind: 'exec', command: 'echo hi' });
    });

    it('a node with a "script" field is treated as script', () => {
      useScriptStore.setState({
        script: JSON.stringify({ nodes: [{ id: 'a', script: 'echo hi' }] }),
      });
      const { result } = renderHook(() => usePipelineScript());
      expect(result.current.steps[0]).toMatchObject({ kind: 'script', script: 'echo hi', shell: 'sh' });
    });

    it('a node with neither field defaults to an empty exec command', () => {
      useScriptStore.setState({ script: JSON.stringify({ nodes: [{ id: 'a' }] }) });
      const { result } = renderHook(() => usePipelineScript());
      expect(result.current.steps[0]).toMatchObject({ kind: 'exec', command: '', args: [] });
    });

    it('an explicit shell other than "bash" falls back to "sh"', () => {
      useScriptStore.setState({
        script: JSON.stringify({ nodes: [{ id: 'a', kind: 'script', script: 's', shell: 'zsh' }] }),
      });
      const { result } = renderHook(() => usePipelineScript());
      expect(result.current.steps[0]).toMatchObject({ shell: 'sh' });
    });
  });

  describe('env parsing', () => {
    it('tolerates a literal entry missing its value', () => {
      useScriptStore.setState({
        script: JSON.stringify({ nodes: [{ id: 'a', command: 'x', env: [{ key: 'K' }] }] }),
      });
      const { result } = renderHook(() => usePipelineScript());
      expect(result.current.steps[0].env).toEqual([{ key: 'K', kind: 'literal', value: '' }]);
    });

    it('tolerates a secret entry missing its secretRef', () => {
      useScriptStore.setState({
        script: JSON.stringify({
          nodes: [{ id: 'a', command: 'x', env: [{ key: 'K', kind: 'secret' }] }],
        }),
      });
      const { result } = renderHook(() => usePipelineScript());
      expect(result.current.steps[0].env).toEqual([{ key: 'K', kind: 'secret', secretRef: '' }]);
    });

    it('a non-array env is treated as empty rather than throwing', () => {
      useScriptStore.setState({
        script: JSON.stringify({ nodes: [{ id: 'a', command: 'x', env: 'nope' }] }),
      });
      const { result } = renderHook(() => usePipelineScript());
      expect(result.current.steps[0].env).toEqual([]);
    });
  });

  describe('handleStepsChange', () => {
    it('seeds a fresh {name, projectId} document when nothing was parsed yet', () => {
      const { result, rerender } = renderHook(() => usePipelineScript({ projectId: 'project-1' }));

      act(() => {
        result.current.handleStepsChange([
          { id: 'build', deps: [], env: [], kind: 'exec', command: 'make', args: [] },
        ]);
      });
      rerender();

      expect(JSON.parse(result.current.script)).toEqual({
        name: 'my-pipeline',
        projectId: 'project-1',
        nodes: [{ id: 'build', deps: [], workingDir: '', env: [], kind: 'exec', command: 'make', args: [] }],
      });
    });

    it('preserves the rest of an already-parsed document (e.g. its name) when steps change', () => {
      const { result, rerender } = renderHook(() => usePipelineScript());
      act(() => result.current.setScript(JSON.stringify({ name: 'my-ci', nodes: [] })));
      rerender();

      act(() => {
        result.current.handleStepsChange([
          { id: 's', deps: [], env: [], kind: 'script', script: 'echo hi', shell: 'bash' },
        ]);
      });
      rerender();

      const parsed = JSON.parse(result.current.script) as { name: string };
      expect(parsed.name).toBe('my-ci');
      // Round-tripped through serializeNode first, which always writes an
      // (empty-string, not absent) workingDir - unlike a hand-authored script
      // that omits the field entirely and parses to `undefined` (see above).
      expect(result.current.steps).toEqual([
        { id: 's', deps: [], workingDir: '', env: [], kind: 'script', script: 'echo hi', shell: 'bash' },
      ]);
    });
  });

  describe('handleNameChange', () => {
    it('is a no-op while nothing has been parsed', () => {
      const { result, rerender } = renderHook(() => usePipelineScript());
      act(() => result.current.handleNameChange('new-name'));
      rerender();
      expect(result.current.script).toBe('');
    });

    it('renames the parsed document once something is parsed', () => {
      const { result, rerender } = renderHook(() => usePipelineScript());
      act(() => result.current.setScript(JSON.stringify({ name: 'old', nodes: [] })));
      rerender();

      act(() => result.current.handleNameChange('new-name'));
      rerender();

      expect(result.current.pipelineName).toBe('new-name');
    });
  });

  describe('isDirty', () => {
    it('is true once the script diverges from the initial baseline, and false again once re-baselined', () => {
      const { result, rerender } = renderHook(() => usePipelineScript());

      act(() => result.current.setScript('{"name":"x","nodes":[]}'));
      rerender();
      expect(result.current.isDirty).toBe(true);

      act(() => result.current.setInitialScript(result.current.script));
      rerender();
      expect(result.current.isDirty).toBe(false);
    });
  });
});
