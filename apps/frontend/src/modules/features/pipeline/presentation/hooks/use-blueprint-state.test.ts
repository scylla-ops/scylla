import { describe, it, expect, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useBlueprintState } from './use-blueprint-state';
import { START_NODE_ID } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';
import type { PipelineStep, ExecPipelineStep } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

const execStep = (id: string, deps: string[] = []): ExecPipelineStep => ({
  id,
  deps,
  kind: 'exec',
  command: 'echo',
  args: [id],
  env: [],
});

const renderBlueprint = (steps: PipelineStep[], pipelineName = 'my-pipeline') => {
  const onStepsChange = vi.fn();
  const utils = renderHook(
    (props: { steps: PipelineStep[]; pipelineName: string }) =>
      useBlueprintState({ ...props, onStepsChange }),
    { initialProps: { steps, pipelineName } },
  );
  return { ...utils, onStepsChange };
};

describe('useBlueprintState — steps -> flow sync', () => {
  it('renders just the start node for an empty pipeline', () => {
    const { result } = renderBlueprint([]);
    expect(result.current.nodes).toHaveLength(1);
    expect(result.current.nodes[0].id).toBe(START_NODE_ID);
    expect(result.current.edges).toHaveLength(0);
  });

  it('renders one node per step, connected from the start node', () => {
    const { result } = renderBlueprint([execStep('a')]);
    expect(result.current.nodes.map(n => n.id).sort()).toEqual(['__start__', 'a']);
    expect(result.current.edges).toContainEqual(
      expect.objectContaining({ source: START_NODE_ID, target: 'a' }),
    );
  });

  it('does not call onStepsChange for an already-clean incoming step list', () => {
    const { onStepsChange } = renderBlueprint([execStep('a'), execStep('b', ['a'])]);
    expect(onStepsChange).not.toHaveBeenCalled();
  });

  it('calls onStepsChange once with the sanitized version when the incoming steps are dirty (duplicate id)', () => {
    const { onStepsChange } = renderBlueprint([execStep('a'), execStep('a')]);
    expect(onStepsChange).toHaveBeenCalledTimes(1);
    expect(onStepsChange.mock.calls[0][0].map((s: PipelineStep) => s.id)).toEqual(['a', 'a_1']);
  });

  it('re-syncs the canvas when the steps prop changes to a genuinely different pipeline', () => {
    const { result, rerender } = renderBlueprint([execStep('a')]);
    rerender({ steps: [execStep('a'), execStep('b', ['a'])], pipelineName: 'my-pipeline' });
    expect(result.current.nodes.map(n => n.id).sort()).toEqual(['__start__', 'a', 'b']);
  });

  it('does not reset the canvas for the steps the hook itself just emitted (no infinite sync loop)', () => {
    // Simulate a controlled parent: it receives onStepsChange and feeds the
    // exact same (sanitized) steps back down as the next `steps` prop.
    const sameRef = { current: null as PipelineStep[] | null };
    const onStepsChange = vi.fn((s: PipelineStep[]) => {
      sameRef.current = s;
    });
    const { result, rerender } = renderHook(
      (props: { steps: PipelineStep[]; pipelineName: string }) =>
        useBlueprintState({ ...props, onStepsChange }),
      { initialProps: { steps: [execStep('a'), execStep('a')] as PipelineStep[], pipelineName: 'p' } },
    );

    expect(onStepsChange).toHaveBeenCalledTimes(1);
    const nodesAfterFirstSync = result.current.nodes;

    rerender({ steps: sameRef.current!, pipelineName: 'p' });

    // Still exactly one emission — the echoed-back steps didn't trigger a second one.
    expect(onStepsChange).toHaveBeenCalledTimes(1);
    expect(result.current.nodes).toBe(nodesAfterFirstSync);
  });
});

describe('useBlueprintState — editing handlers', () => {
  it('handleConnect adds an edge and emits the new dependency', () => {
    const { result } = renderBlueprint([execStep('a'), execStep('b')]);
    act(() => result.current.handleConnect({ source: 'a', target: 'b', sourceHandle: null, targetHandle: null }));

    expect(result.current.edges).toContainEqual(expect.objectContaining({ source: 'a', target: 'b' }));
  });

  it('handleConnect ignores a connection targeting the start node', () => {
    const { result } = renderBlueprint([execStep('a')]);
    const edgesBefore = result.current.edges;
    act(() =>
      result.current.handleConnect({
        source: 'a',
        target: START_NODE_ID,
        sourceHandle: null,
        targetHandle: null,
      }),
    );
    expect(result.current.edges).toBe(edgesBefore);
  });

  it('handleEdgesDelete removes the edge and the corresponding dependency', () => {
    const { result, onStepsChange } = renderBlueprint([execStep('a'), execStep('b', ['a'])]);
    const edgeAtoB = result.current.edges.find(e => e.source === 'a' && e.target === 'b')!;

    act(() => result.current.handleEdgesDelete([edgeAtoB]));

    const emitted = onStepsChange.mock.calls.at(-1)![0] as PipelineStep[];
    expect(emitted.find(s => s.id === 'b')?.deps).toEqual([]);
  });

  it('handleNodesDelete removes the node and any edge touching it', () => {
    const { result, onStepsChange } = renderBlueprint([execStep('a'), execStep('b', ['a'])]);
    const nodeA = result.current.nodes.find(n => n.id === 'a')!;

    act(() => result.current.handleNodesDelete([nodeA]));

    expect(result.current.nodes.map(n => n.id)).not.toContain('a');
    expect(result.current.edges.some(e => e.source === 'a' || e.target === 'a')).toBe(false);
    const emitted = onStepsChange.mock.calls.at(-1)![0] as PipelineStep[];
    expect(emitted.map(s => s.id)).toEqual(['b']);
    expect(emitted[0].deps).toEqual([]); // dangling dep on the deleted node dropped too
  });

  it('handleAddNode adds a node with a unique id and emits the new step', () => {
    const { result, onStepsChange } = renderBlueprint([execStep('a')]);

    act(() =>
      result.current.handleAddNode('a', { kind: 'exec', command: 'ls', args: [], env: [] }),
    );

    // "a" was taken, so the new node gets a disambiguated id.
    const ids = result.current.nodes.map(n => n.id);
    expect(ids).toContain('a_1');
    const emitted = onStepsChange.mock.calls.at(-1)![0] as PipelineStep[];
    expect(emitted.map(s => s.id).sort()).toEqual(['a', 'a_1']);
  });

  it('handleEditNode renames a node and remaps dependent nodes\' deps', () => {
    const { result, onStepsChange } = renderBlueprint([execStep('build'), execStep('test', ['build'])]);

    act(() =>
      result.current.handleEditNode('build', 'compile', {
        kind: 'exec',
        command: 'echo',
        args: [],
        env: [],
      }),
    );

    expect(result.current.nodes.map(n => n.id).sort()).toEqual(['__start__', 'compile', 'test']);
    const emitted = onStepsChange.mock.calls.at(-1)![0] as PipelineStep[];
    expect(emitted.find(s => s.id === 'test')?.deps).toEqual(['compile']);
  });

  it('handleEditNode remaps the edges touching the renamed node too', () => {
    const { result } = renderBlueprint([execStep('build'), execStep('test', ['build'])]);

    act(() =>
      result.current.handleEditNode('build', 'compile', {
        kind: 'exec',
        command: 'echo',
        args: [],
        env: [],
      }),
    );

    expect(
      result.current.edges.some(e => e.source === 'compile' && e.target === 'test'),
    ).toBe(true);
    expect(result.current.edges.some(e => e.source === 'build' || e.target === 'build')).toBe(
      false,
    );
  });
});
