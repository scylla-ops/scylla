import { describe, it, expect } from 'vitest';
import {
  sanitizeSteps,
  stepsToFlow,
  flowToSteps,
  generateUniqueNodeId,
  START_NODE_ID,
} from './blueprint-converter';
import type { PipelineStep, ExecPipelineStep } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

const execStep = (id: string, deps: string[] = []): ExecPipelineStep => ({
  id,
  deps,
  kind: 'exec',
  command: 'echo',
  args: [id],
  env: [],
});

describe('sanitizeSteps', () => {
  it('leaves already-valid steps untouched', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('b', ['a'])];
    expect(sanitizeSteps(steps)).toEqual(steps);
  });

  it('deduplicates a repeated id by suffixing _1, _2, ...', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('a'), execStep('a')];
    const result = sanitizeSteps(steps);
    expect(result.map(s => s.id)).toEqual(['a', 'a_1', 'a_2']);
  });

  it('remaps a dependency on a duplicated id to the LAST occurrence, not the first', () => {
    // Two steps both start out as "build": the first keeps id "build", the
    // second is renamed to "build_1". The id->id rename map is keyed by the
    // ORIGINAL id, so the second occurrence's entry overwrites the first's —
    // a downstream dep on "build" ends up pointing at "build_1". Documented
    // here as the actual (if surprising) tie-break, not a first-wins one.
    const steps: PipelineStep[] = [execStep('build'), execStep('build'), execStep('test', ['build'])];
    const result = sanitizeSteps(steps);
    const ids = result.map(s => s.id);
    expect(ids).toEqual(['build', 'build_1', 'test']);
    expect(result.find(s => s.id === 'test')?.deps).toEqual(['build_1']);
  });

  it('drops a self-dependency', () => {
    const steps: PipelineStep[] = [execStep('a', ['a'])];
    expect(sanitizeSteps(steps)[0].deps).toEqual([]);
  });

  it('drops a dependency pointing at a non-existent node', () => {
    const steps: PipelineStep[] = [execStep('a', ['ghost'])];
    expect(sanitizeSteps(steps)[0].deps).toEqual([]);
  });

  it('keeps a valid dependency alongside a dropped invalid one', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('b', ['a', 'ghost', 'b'])];
    expect(sanitizeSteps(steps)[1].deps).toEqual(['a']);
  });
});

describe('stepsToFlow', () => {
  it('produces one start node plus one node per step', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('b', ['a'])];
    const { nodes } = stepsToFlow(steps, 'my-pipeline');
    expect(nodes).toHaveLength(3);
    expect(nodes[0].id).toBe(START_NODE_ID);
    expect(nodes[0].data).toEqual({ name: 'my-pipeline' });
  });

  it('wires an edge from the start node to every root step (no deps)', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('b')];
    const { edges } = stepsToFlow(steps, 'p');
    const fromStart = edges.filter(e => e.source === START_NODE_ID).map(e => e.target);
    expect(fromStart.sort()).toEqual(['a', 'b']);
  });

  it('wires a direct edge for each dependency, not from the start node', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('b', ['a'])];
    const { edges } = stepsToFlow(steps, 'p');
    expect(edges).toContainEqual(expect.objectContaining({ source: 'a', target: 'b' }));
    // "b" has a real dependency, so it must NOT also get a start-node edge
    expect(edges.some(e => e.source === START_NODE_ID && e.target === 'b')).toBe(false);
  });

  it('places steps at increasing depth (x position) the further they are from a root', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('b', ['a']), execStep('c', ['b'])];
    const { nodes } = stepsToFlow(steps, 'p');
    const byId = Object.fromEntries(nodes.map(n => [n.id, n]));
    expect(byId.a.position.x).toBeLessThan(byId.b.position.x);
    expect(byId.b.position.x).toBeLessThan(byId.c.position.x);
  });

  it('gives siblings at the same depth distinct y positions', () => {
    const steps: PipelineStep[] = [execStep('a'), execStep('b', ['a']), execStep('c', ['a'])];
    const { nodes } = stepsToFlow(steps, 'p');
    const byId = Object.fromEntries(nodes.map(n => [n.id, n]));
    expect(byId.b.position.y).not.toBe(byId.c.position.y);
  });

  it('runs sanitizeSteps first, so a bad incoming graph still produces a coherent flow', () => {
    // duplicate id "a" and a dangling dep — must not throw, must still connect
    const steps: PipelineStep[] = [execStep('a'), execStep('a'), execStep('c', ['ghost'])];
    const { nodes, edges, sanitizedSteps } = stepsToFlow(steps, 'p');
    expect(nodes).toHaveLength(4); // start + 3 sanitized steps
    expect(sanitizedSteps.map(s => s.id)).toEqual(['a', 'a_1', 'c']);
    // "c"'s dangling dep was dropped, so it's a root and gets a start-node edge
    expect(edges).toContainEqual(expect.objectContaining({ source: START_NODE_ID, target: 'c' }));
  });

  it('handles an empty pipeline (just the start node, no edges)', () => {
    const { nodes, edges } = stepsToFlow([], 'empty');
    expect(nodes).toHaveLength(1);
    expect(nodes[0].id).toBe(START_NODE_ID);
    expect(edges).toHaveLength(0);
  });
});

describe('flowToSteps', () => {
  it('rebuilds steps from nodes and edges, ignoring the start node', () => {
    const { nodes, edges } = stepsToFlow([execStep('a'), execStep('b', ['a'])], 'p');
    const rebuilt = flowToSteps(nodes, edges);
    expect(rebuilt).toHaveLength(2);
    expect(rebuilt.find(s => s.id === 'b')?.deps).toEqual(['a']);
  });

  it('never derives a dependency on the start node itself', () => {
    const { nodes, edges } = stepsToFlow([execStep('a')], 'p');
    const rebuilt = flowToSteps(nodes, edges);
    expect(rebuilt[0].deps).toEqual([]);
  });

  it('round-trips exec steps through stepsToFlow -> flowToSteps unchanged', () => {
    const steps: PipelineStep[] = [
      { id: 'a', deps: [], kind: 'exec', command: 'echo', args: ['hi'], env: [] },
      { id: 'b', deps: ['a'], kind: 'exec', command: 'ls', args: [], workingDir: '/tmp', env: [] },
    ];
    const { nodes, edges } = stepsToFlow(steps, 'p');
    expect(flowToSteps(nodes, edges)).toEqual(steps);
  });

  it('round-trips script steps through stepsToFlow -> flowToSteps unchanged', () => {
    const steps: PipelineStep[] = [
      { id: 'build', deps: [], kind: 'script', script: 'echo hi', shell: 'bash', env: [] },
    ];
    const { nodes, edges } = stepsToFlow(steps, 'p');
    expect(flowToSteps(nodes, edges)).toEqual(steps);
  });
});

describe('generateUniqueNodeId', () => {
  it('returns the desired id when it is free', () => {
    expect(generateUniqueNodeId('build', new Set())).toBe('build');
  });

  it('suffixes with _1 when the desired id is taken', () => {
    expect(generateUniqueNodeId('build', new Set(['build']))).toBe('build_1');
  });

  it('keeps incrementing the suffix past existing collisions', () => {
    expect(generateUniqueNodeId('build', new Set(['build', 'build_1', 'build_2']))).toBe(
      'build_3',
    );
  });

  it('excludes the node being renamed, so renaming to its own current id is a no-op', () => {
    expect(generateUniqueNodeId('build', new Set(['build']), 'build')).toBe('build');
  });
});
