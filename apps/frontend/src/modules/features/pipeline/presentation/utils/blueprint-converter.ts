import type { Node, Edge, MarkerType } from 'reactflow';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export interface PipelineNodeData {
  id: string;
  command: string;
  args: string[];
  deps: string[];
}

export interface StartNodeData {
  name: string;
}

export const START_NODE_ID = '__start__';
export const EDGE_COLOR = 'oklch(65.752% 0.25 180)';

const NODE_WIDTH = 260;
const NODE_HEIGHT = 120;
const HORIZONTAL_GAP = 100;
const VERTICAL_GAP = 60;
const START_NODE_OFFSET = NODE_WIDTH + HORIZONTAL_GAP;

export const DEFAULT_EDGE_STYLE = {
  animated: true,
  type: 'deletable' as const,
  markerEnd: { type: 'arrowclosed' as MarkerType, color: EDGE_COLOR },
  style: { stroke: EDGE_COLOR, strokeWidth: 2 },
};

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
    deps: s.deps
      .map(d => idMap.get(d) ?? d)
      .filter(d => d !== s.id && validIds.has(d)),
  }));
}

export function stepsToFlow(
  rawSteps: PipelineStep[],
  pipelineName: string,
): { nodes: Node[]; edges: Edge[]; sanitizedSteps: PipelineStep[] } {
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

  const startNode: Node<StartNodeData> = {
    id: START_NODE_ID,
    type: 'startNode',
    position: { x: 0, y: totalHeight / 2 - 30 },
    data: { name: pipelineName },
    deletable: false,
  };

  const stepNodes: Node<PipelineNodeData>[] = steps.map(step => {
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
      data: {
        id: step.id,
        command: step.command,
        args: step.args,
        deps: step.deps,
      },
    };
  });

  const stepEdges: Edge[] = steps.flatMap(step =>
    step.deps.map(dep => ({
      id: `${dep}->${step.id}`,
      source: dep,
      target: step.id,
      ...DEFAULT_EDGE_STYLE,
    })),
  );

  const roots = steps.filter(s => s.deps.length === 0);
  const startEdges: Edge[] = roots.map(root => ({
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

export function flowToSteps(nodes: Node[], edges: Edge[]): PipelineStep[] {
  return nodes
    .filter(node => node.id !== START_NODE_ID)
    .map(node => {
      const data = node.data as PipelineNodeData;
      const incomingDeps = edges
        .filter(e => e.target === node.id && e.source !== START_NODE_ID)
        .map(e => e.source);
      return {
        id: data.id,
        deps: incomingDeps,
        command: data.command,
        args: data.args,
      };
    });
}

/**
 * Generate a unique node ID, avoiding collisions with existing IDs.
 * Optionally exclude one ID (useful when renaming a node).
 */
export function generateUniqueNodeId(desired: string, existingIds: Set<string>, excludeId?: string): string {
  const ids = new Set(existingIds);
  if (excludeId) ids.delete(excludeId);
  if (!ids.has(desired)) return desired;
  let i = 1;
  while (ids.has(`${desired}_${i}`)) i++;
  return `${desired}_${i}`;
}
