import type { Edge, Node } from '@xyflow/svelte';
import type {
  ExecPipelineStep,
  PipelineStep,
  ScriptPipelineStep,
} from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

/**
 * The single translation between the canvas graph and `PipelineStep[]`.
 *
 * Pure, and the safety net of the Phase 5 port: `reactflow` became
 * `@xyflow/svelte`, and the only thing that changed here is which package the
 * `Node` / `Edge` types come from — every rule below, and every test beside it,
 * is the code that did not have to be rewritten.
 *
 * One shape did change. A node's `data` must satisfy `Record<string, unknown>`,
 * and `PipelineStep` is a union of *interfaces*, which TypeScript gives no
 * implicit index signature. So a step node carries `{ step }` rather than being
 * the step, and `node.data.step` replaces the `node.data as PipelineNodeData`
 * cast the React version needed at every use site — a cast fewer, not more.
 */

export type StepNodeData = { step: PipelineStep };
export type StartNodeData = { name: string };

/**
 * A step as the node dialog builds it: everything but its identity and its
 * wiring, both of which the canvas owns.
 */
export type NodeFormValue =
  | Omit<ExecPipelineStep, 'id' | 'deps'>
  | Omit<ScriptPipelineStep, 'id' | 'deps'>;

export type BlueprintStepNode = Node<StepNodeData, 'pipelineStep'>;
export type BlueprintStartNode = Node<StartNodeData, 'startNode'>;
export type BlueprintNode = BlueprintStepNode | BlueprintStartNode;
export type BlueprintEdge = Edge;

export const START_NODE_ID = '__start__';
export const EDGE_COLOR = 'var(--primary)';

const NODE_WIDTH = 260;
const NODE_HEIGHT = 120;
const HORIZONTAL_GAP = 100;
const VERTICAL_GAP = 60;
const START_NODE_OFFSET = NODE_WIDTH + HORIZONTAL_GAP;

/**
 * `style` is a CSS string here, where reactflow took an object — the one
 * cosmetic difference between the two libraries this file touches.
 */
export const DEFAULT_EDGE_STYLE = {
  animated: true,
  type: 'deletable',
  markerEnd: { type: 'arrowclosed', color: EDGE_COLOR },
  style: `stroke: ${EDGE_COLOR}; stroke-width: 2;`,
} satisfies Partial<BlueprintEdge>;

/**
 * Compute the depth (column) of each node via BFS from roots.
 */
function computeDepths(steps: PipelineStep[]): Map<string, number> {
  const depthMap = new Map<string, number>();
  const childrenOf = new Map<string, string[]>();

  for (const step of steps) {
    childrenOf.set(step.id, []);
  }
  for (const step of steps) {
    for (const dep of step.deps) {
      childrenOf.get(dep)?.push(step.id);
    }
  }

  const roots = steps.filter(s => s.deps.length === 0);
  const queue: { id: string; depth: number }[] = roots.map(r => ({ id: r.id, depth: 0 }));

  while (queue.length > 0) {
    const { id, depth } = queue.shift()!;
    const current = depthMap.get(id);
    if (current !== undefined && current >= depth) continue;
    depthMap.set(id, depth);
    for (const child of childrenOf.get(id) ?? []) {
      queue.push({ id: child, depth: depth + 1 });
    }
  }

  for (const step of steps) {
    if (!depthMap.has(step.id)) {
      depthMap.set(step.id, 0);
    }
  }

  return depthMap;
}

/**
 * Sanitize steps: deduplicate IDs, remove self-deps, remove deps to non-existent nodes.
 */
export function sanitizeSteps(steps: PipelineStep[]): PipelineStep[] {
  const seen = new Set<string>();
  const idMap = new Map<string, string>();
  const deduped: PipelineStep[] = [];

  for (const step of steps) {
    let id = step.id;
    if (seen.has(id)) {
      let i = 1;
      while (seen.has(`${step.id}_${i}`)) i++;
      id = `${step.id}_${i}`;
    }
    seen.add(id);
    idMap.set(step.id, id);
    deduped.push({ ...step, id });
  }

  const validIds = new Set(deduped.map(s => s.id));

  return deduped.map(s => ({
    ...s,
    deps: s.deps.map(d => idMap.get(d) ?? d).filter(d => d !== s.id && validIds.has(d)),
  }));
}

export function stepsToFlow(
  rawSteps: PipelineStep[],
  pipelineName: string,
): { nodes: BlueprintNode[]; edges: BlueprintEdge[]; sanitizedSteps: PipelineStep[] } {
  const steps = sanitizeSteps(rawSteps);
  const depthMap = computeDepths(steps);

  const depthGroups = new Map<number, PipelineStep[]>();
  for (const step of steps) {
    const d = depthMap.get(step.id) ?? 0;
    if (!depthGroups.has(d)) depthGroups.set(d, []);
    depthGroups.get(d)!.push(step);
  }

  const maxGroupSize = Math.max(1, ...Array.from(depthGroups.values()).map(g => g.length));
  const totalHeight = maxGroupSize * (NODE_HEIGHT + VERTICAL_GAP) - VERTICAL_GAP;

  const startNode: BlueprintStartNode = {
    id: START_NODE_ID,
    type: 'startNode',
    position: { x: 0, y: totalHeight / 2 - 30 },
    data: { name: pipelineName },
    deletable: false,
  };

  const stepNodes: BlueprintStepNode[] = steps.map(step => {
    const depth = depthMap.get(step.id) ?? 0;
    const group = depthGroups.get(depth) ?? [step];
    const indexInGroup = group.indexOf(step);

    return {
      id: step.id,
      type: 'pipelineStep',
      position: {
        x: START_NODE_OFFSET + depth * (NODE_WIDTH + HORIZONTAL_GAP),
        y: indexInGroup * (NODE_HEIGHT + VERTICAL_GAP),
      },
      data: { step },
    };
  });

  const stepEdges: BlueprintEdge[] = steps.flatMap(step =>
    step.deps.map(dep => ({
      id: `${dep}->${step.id}`,
      source: dep,
      target: step.id,
      ...DEFAULT_EDGE_STYLE,
    })),
  );

  const roots = steps.filter(s => s.deps.length === 0);
  const startEdges: BlueprintEdge[] = roots.map(root => ({
    id: `${START_NODE_ID}->${root.id}`,
    source: START_NODE_ID,
    target: root.id,
    ...DEFAULT_EDGE_STYLE,
  }));

  return {
    nodes: [startNode, ...stepNodes],
    edges: [...startEdges, ...stepEdges],
    sanitizedSteps: steps,
  };
}

/** A step node's payload, or `undefined` for the start node. */
export const stepOf = (node: BlueprintNode): PipelineStep | undefined =>
  node.type === 'pipelineStep' ? node.data.step : undefined;

export function flowToSteps(nodes: BlueprintNode[], edges: BlueprintEdge[]): PipelineStep[] {
  return nodes.flatMap(node => {
    const step = stepOf(node);
    if (!step) return [];

    const deps = edges
      .filter(edge => edge.target === node.id && edge.source !== START_NODE_ID)
      .map(edge => edge.source);

    const base = { id: step.id, deps, workingDir: step.workingDir, env: step.env };

    return [
      step.kind === 'script'
        ? { ...base, kind: 'script' as const, script: step.script, shell: step.shell }
        : { ...base, kind: 'exec' as const, command: step.command, args: step.args },
    ];
  });
}

/**
 * Generate a unique node ID, avoiding collisions with existing IDs.
 * Optionally exclude one ID (useful when renaming a node).
 */
export function generateUniqueNodeId(
  desired: string,
  existingIds: Set<string>,
  excludeId?: string,
): string {
  const ids = new Set(existingIds);
  if (excludeId) ids.delete(excludeId);
  if (!ids.has(desired)) return desired;
  let i = 1;
  while (ids.has(`${desired}_${i}`)) i++;
  return `${desired}_${i}`;
}
