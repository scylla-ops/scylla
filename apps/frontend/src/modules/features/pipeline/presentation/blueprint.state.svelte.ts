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
  /** Getters: the steps and the name are the editor's document, and it changes. */
  steps: () => PipelineStep[];
  pipelineName: () => string;
  onStepsChange: (steps: PipelineStep[]) => void;
}

/**
 * The canvas graph: nodes and edges, kept in step with the script document.
 *
 * Two sources of truth face each other and neither derives from the other. The
 * script owns the steps; the canvas owns their *positions*, which no script
 * records — so the graph cannot be a `$derived` of the steps, or every node
 * would snap back to its computed column the moment anything changed.
 *
 * Hence the guard, carried over verbatim from `use-blueprint-state.ts`:
 * `lastEmitted` records the document this state itself just produced, so the
 * echo coming back down as a new `steps` prop is recognised and ignored. Take
 * it out and a single edit loops — steps → flow → steps → flow.
 *
 * **What the port removed is most of the handlers.** `@xyflow/svelte` takes
 * `nodes` and `edges` as bindable props and writes an added or deleted element
 * into them *before* firing `onconnect` / `ondelete`, so the five reducers the
 * React hook maintained — each juggling two nested `setState` callbacks to read
 * the other array — collapse into {@link sync}. Only the two operations the
 * library knows nothing about, adding and editing a step through the dialog,
 * are still written here.
 */
export const createBlueprintState = (params: BlueprintStateParams) => {
  // `$state.raw`: these arrays are replaced wholesale, never mutated in place,
  // and a deep proxy over every node would cost for nothing.
  let nodes = $state.raw<BlueprintNode[]>([]);
  let edges = $state.raw<BlueprintEdge[]>([]);

  /** The document this state last handed upwards, so its echo is recognised. */
  let lastEmitted = '';

  const keyOf = (steps: PipelineStep[], name: string) => JSON.stringify({ steps, name });

  const emit = () => {
    const steps = flowToSteps(nodes, edges);
    const key = keyOf(steps, params.pipelineName());
    if (key === lastEmitted) return;

    lastEmitted = key;
    params.onStepsChange(steps);
  };

  // Synchronising with the document, which lives outside this state — the one
  // thing `$effect` is for. It reads `steps` and `pipelineName` and writes only
  // `nodes` / `edges`, so it never re-triggers on its own output.
  $effect(() => {
    const steps = params.steps();
    const name = params.pipelineName();
    if (keyOf(steps, name) === lastEmitted) return;

    const flow = stepsToFlow(steps, name);
    nodes = flow.nodes;
    edges = flow.edges;

    lastEmitted = keyOf(flow.sanitizedSteps, name);
    // Sanitising is a change the document has not seen — a duplicate id was
    // renamed, a dangling dependency dropped — so it has to travel back up.
    if (JSON.stringify(flow.sanitizedSteps) !== JSON.stringify(steps)) {
      params.onStepsChange(flow.sanitizedSteps);
    }
  });

  return {
    get nodes() {
      return nodes;
    },
    /** Written by `bind:nodes` — dragging, selection and deletion happen there. */
    set nodes(next: BlueprintNode[]) {
      nodes = next;
    },
    get edges() {
      return edges;
    },
    set edges(next: BlueprintEdge[]) {
      edges = next;
    },

    /**
     * Whether a hand-drawn edge may exist at all.
     *
     * Nothing may depend on the start node, which stands for the pipeline
     * itself: an edge into it would mean a step the pipeline waits for before
     * it begins. Answered *before* the edge is created, so a refused connection
     * never reaches the document.
     */
    canConnect: (connection: Connection) => connection.target !== START_NODE_ID,

    /** The canvas changed the graph itself — publish what it now says. */
    sync: emit,

    /** Adds a step, disambiguating its id against the ones already on the canvas. */
    addNode(nodeId: string, value: NodeFormValue) {
      const id = generateUniqueNodeId(nodeId, new Set(nodes.map(node => node.id)));

      const node: BlueprintStepNode = {
        id,
        type: 'pipelineStep',
        // Offset at random so a second node added without touching the canvas
        // does not land exactly on top of the first.
        position: { x: 400 + Math.random() * 200, y: Math.random() * 300 },
        data: { step: { id, deps: [], ...value } },
      };

      nodes = [...nodes, node];
      emit();
    },

    /**
     * Rewrites a step, id included.
     *
     * A rename has to be followed everywhere the old id is written down — the
     * `deps` of every other step, and both ends of every edge — or the step
     * silently loses its wiring on the next round trip through the document.
     */
    editNode(originalId: string, newNodeId: string, value: NodeFormValue) {
      const id = generateUniqueNodeId(newNodeId, new Set(nodes.map(node => node.id)), originalId);

      nodes = nodes.map((node): BlueprintNode => {
        // The start node is neither renameable nor a dependant — it is the
        // pipeline, not a step — so it is out of this entirely.
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
