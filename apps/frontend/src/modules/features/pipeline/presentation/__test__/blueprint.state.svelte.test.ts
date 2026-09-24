import { describe, it, expect, vi } from 'vitest';
import { flushSync } from 'svelte';
import type { ExecPipelineStep, PipelineStep } from '../../domain/structs/pipeline.struct.ts';
import { createBlueprintState } from '../blueprint.state.svelte.ts';
import { START_NODE_ID, flowToSteps } from '../utils/blueprint-converter.ts';

const execStep = (id: string, deps: string[] = []): ExecPipelineStep => ({
  id,
  deps,
  kind: 'exec',
  command: 'echo',
  args: [id],
  env: [],
});

const withState = (
  initial: PipelineStep[],
  run: (state: ReturnType<typeof createBlueprintState>, onStepsChange: ReturnType<typeof vi.fn>) => void,
) => {
  const onStepsChange = vi.fn();
  const cleanup = $effect.root(() => {
    const state = createBlueprintState({
      steps: () => initial,
      pipelineName: () => 'nightly',
      onStepsChange,
    });
    flushSync();
    run(state, onStepsChange);
  });
  cleanup();
};

describe('createBlueprintState', () => {
  it('builds the graph from the steps, with the start node', () => {
    withState([execStep('build'), execStep('test', ['build'])], state => {
      expect(state.nodes.map(node => node.id).sort()).toEqual([START_NODE_ID, 'build', 'test'].sort());
      expect(state.edges.some(edge => edge.source === 'build' && edge.target === 'test')).toBe(true);
    });
  });

  it('does not publish valid steps back to the document', () => {
    withState([execStep('build')], (_state, onStepsChange) => {
      expect(onStepsChange).not.toHaveBeenCalled();
    });
  });

  it('publishes the steps again when it had to repair them', () => {
    withState([execStep('build', ['ghost'])], (_state, onStepsChange) => {
      expect(onStepsChange).toHaveBeenCalledWith([execStep('build')]);
    });
  });

  it('refuses an edge into the start node', () => {
    withState([execStep('build')], state => {
      expect(state.canConnect({ source: 'build', target: START_NODE_ID } as never)).toBe(false);
      expect(state.canConnect({ source: START_NODE_ID, target: 'build' } as never)).toBe(true);
    });
  });

  it('adds a step with a unique id and publishes it', () => {
    withState([execStep('build')], (state, onStepsChange) => {
      state.addNode('build', { kind: 'exec', command: 'make', args: [], env: [] });

      const published = onStepsChange.mock.lastCall?.[0] as PipelineStep[];
      expect(published.map(step => step.id)).toEqual(['build', 'build_1']);
    });
  });

  it('renames a step everywhere its id is written', () => {
    withState([execStep('build'), execStep('test', ['build'])], (state, onStepsChange) => {
      state.editNode('build', 'compile', { kind: 'exec', command: 'make', args: [], env: [] });

      const published = onStepsChange.mock.lastCall?.[0] as PipelineStep[];
      expect(published.find(step => step.id === 'compile')).toMatchObject({ command: 'make' });
      expect(published.find(step => step.id === 'test')?.deps).toEqual(['compile']);
      expect(state.edges.some(edge => edge.source === 'compile' && edge.target === 'test')).toBe(true);
    });
  });

  it('publishes the graph when the canvas changes it', () => {
    withState([execStep('build'), execStep('test', ['build'])], (state, onStepsChange) => {
      state.edges = state.edges.filter(edge => edge.target !== 'test');
      state.sync();

      expect(onStepsChange).toHaveBeenLastCalledWith(flowToSteps(state.nodes, state.edges));
      expect(
        (onStepsChange.mock.lastCall?.[0] as PipelineStep[]).find(step => step.id === 'test')?.deps,
      ).toEqual([]);
    });
  });

  it('does not publish the same document twice', () => {
    withState([execStep('build')], (state, onStepsChange) => {
      state.sync();
      const calls = onStepsChange.mock.calls.length;
      state.sync();

      expect(onStepsChange.mock.calls.length).toBe(calls);
    });
  });
});
