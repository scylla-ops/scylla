import type { Connection } from '@xyflow/svelte';
import type { PipelineStep } from '../domain/structs/pipeline.struct.ts';
import {
  START_NODE_ID,
  flowToSteps,
  generateUniqueNodeId,
  stepsToFlow,
  type BlueprintEdge,
  type BlueprintNode,
  type BlueprintStepNode,
  type NodeFormValue,
} from './utils/blueprint-converter.ts';

export interface BlueprintStateParams {
  /** Getters: the editor's document changes. */
  steps: () => PipelineStep[];
  pipelineName: () => string;
  onStepsChange: (steps: PipelineStep[]) => void;
}

/**
 * The canvas graph, in step with the script. The script owns the steps, the
 * canvas owns the positions, so the graph cannot be derived from the steps.
 * `lastEmitted` recognises the echo of our own edits, or one edit would loop.
 * `@xyflow/svelte` writes added and deleted elements itself: `sync` publishes them.
 */
export const createBlueprintState = (params: BlueprintStateParams) => {
  // `$state.raw`: replaced whole, never mutated.
  let nodes = $state.raw<BlueprintNode[]>([]);
  let edges = $state.raw<BlueprintEdge[]>([]);

  let lastEmitted = '';

  const keyOf = (steps: PipelineStep[], name: string) => JSON.stringify({ steps, name });

  const emit = () => {
    const steps = flowToSteps(nodes, edges);
    const key = keyOf(steps, params.pipelineName());
    if (key === lastEmitted) return;

    lastEmitted = key;
    params.onStepsChange(steps);
  };

  // Writes only `nodes` and `edges`, so it never re-triggers itself.
  $effect(() => {
    const steps = params.steps();
    const name = params.pipelineName();
    if (keyOf(steps, name) === lastEmitted) return;

    const flow = stepsToFlow(steps, name);
    nodes = flow.nodes;
    edges = flow.edges;

    lastEmitted = keyOf(flow.sanitizedSteps, name);
    // Sanitising (renamed duplicate id, dropped dangling dependency) must go back up.
    if (JSON.stringify(flow.sanitizedSteps) !== JSON.stringify(steps)) {
      params.onStepsChange(flow.sanitizedSteps);
    }
  });

  return {
    get nodes() {
      return nodes;
    },
    set nodes(next: BlueprintNode[]) {
      nodes = next;
    },
    get edges() {
      return edges;
    },
    set edges(next: BlueprintEdge[]) {
      edges = next;
    },

    /** Nothing may depend on the start node: it is the pipeline itself. */
    canConnect: (connection: Connection) => connection.target !== START_NODE_ID,

    sync: emit,

    addNode(nodeId: string, value: NodeFormValue) {
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- a lookup table local to this call, never state
      const id = generateUniqueNodeId(nodeId, new Set(nodes.map(node => node.id)));

      const node: BlueprintStepNode = {
        id,
        type: 'pipelineStep',
        // Random offset, so two added nodes do not stack.
        position: { x: 400 + Math.random() * 200, y: Math.random() * 300 },
        data: { step: { id, deps: [], ...value } },
      };

      nodes = [...nodes, node];
      emit();
    },

    /** A rename is followed in the `deps` of the other steps and in the edges. */
    editNode(originalId: string, newNodeId: string, value: NodeFormValue) {
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- a lookup table local to this call, never state
      const id = generateUniqueNodeId(newNodeId, new Set(nodes.map(node => node.id)), originalId);

      nodes = nodes.map((node): BlueprintNode => {
        // The start node is the pipeline, not a step.
        if (node.type !== 'pipelineStep') return node;

        if (node.id === originalId) {
          return { ...node, id, data: { step: { id, deps: [], ...value } } };
        }

        const deps = node.data.step.deps;
        if (!deps.includes(originalId)) return node;

        return {
          ...node,
          data: {
            step: { ...node.data.step, deps: deps.map(dep => (dep === originalId ? id : dep)) },
          },
        };
      });

      if (originalId !== id) {
        edges = edges.map(edge => {
          if (edge.source !== originalId && edge.target !== originalId) return edge;

          const source = edge.source === originalId ? id : edge.source;
          const target = edge.target === originalId ? id : edge.target;
          return { ...edge, id: `${source}->${target}`, source, target };
        });
      }

      emit();
    },
  };
};

export type BlueprintState = ReturnType<typeof createBlueprintState>;
