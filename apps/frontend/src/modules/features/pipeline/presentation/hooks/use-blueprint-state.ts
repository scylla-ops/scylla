import { useCallback, useEffect, useRef } from 'react';
import {
  addEdge,
  type Connection,
  type Edge,
  type Node,
  useEdgesState,
  useNodesState,
} from 'reactflow';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import {
  DEFAULT_EDGE_STYLE,
  flowToSteps,
  generateUniqueNodeId,
  type PipelineNodeData,
  START_NODE_ID,
  stepsToFlow,
} from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';

interface UseBlueprintStateParams {
  steps: PipelineStep[];
  pipelineName: string;
  onStepsChange: (steps: PipelineStep[]) => void;
}

export function useBlueprintState({ steps, pipelineName, onStepsChange }: UseBlueprintStateParams) {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  // Guard to prevent infinite sync loop: steps → flow → steps
  const lastEmittedStepsRef = useRef<string>('');

  const emitChange = useCallback(
    (updatedNodes: Node[], updatedEdges: Edge[]) => {
      const newSteps = flowToSteps(updatedNodes, updatedEdges);
      const serialized = JSON.stringify(newSteps);
      if (serialized !== lastEmittedStepsRef.current) {
        lastEmittedStepsRef.current = serialized;
        onStepsChange(newSteps);
      }
    },
    [onStepsChange],
  );

  // ── Sync: steps → flow (external source of truth → canvas) ──
  useEffect(() => {
    const serialized = JSON.stringify(steps);
    if (serialized === lastEmittedStepsRef.current) return;

    const { nodes: n, edges: e, sanitizedSteps } = stepsToFlow(steps, pipelineName);
    setNodes(n);
    setEdges(e);

    const sanitizedSerialized = JSON.stringify(sanitizedSteps);
    if (sanitizedSerialized !== serialized) {
      lastEmittedStepsRef.current = sanitizedSerialized;
      onStepsChange(sanitizedSteps);
    }
  }, [steps, pipelineName, setNodes, setEdges, onStepsChange]);

  // ── Connection handler ──
  const handleConnect = useCallback(
    (connection: Connection) => {
      if (connection.target === START_NODE_ID) return;

      setEdges(currentEdges => {
        const updatedEdges = addEdge(
          { ...connection, ...DEFAULT_EDGE_STYLE },
          currentEdges,
        );
        // Read latest nodes to emit change
        setNodes(currentNodes => {
          emitChange(currentNodes, updatedEdges);
          return currentNodes;
        });
        return updatedEdges;
      });
    },
    [setEdges, setNodes, emitChange],
  );

  // ── Delete edges handler ──
  const handleEdgesDelete = useCallback(
    (deletedEdges: Edge[]) => {
      const deletedIds = new Set(deletedEdges.map(e => e.id));

      setEdges(currentEdges => {
        const remainingEdges = currentEdges.filter(e => !deletedIds.has(e.id));
        setNodes(currentNodes => {
          emitChange(currentNodes, remainingEdges);
          return currentNodes;
        });
        return remainingEdges;
      });
    },
    [setEdges, setNodes, emitChange],
  );

  // ── Delete nodes handler ──
  const handleNodesDelete = useCallback(
    (deletedNodes: Node[]) => {
      const deletedIds = new Set(deletedNodes.map(n => n.id));

      setNodes(currentNodes => {
        const remainingNodes = currentNodes.filter(n => !deletedIds.has(n.id));
        setEdges(currentEdges => {
          const remainingEdges = currentEdges.filter(
            e => !deletedIds.has(e.source) && !deletedIds.has(e.target),
          );
          emitChange(remainingNodes, remainingEdges);
          return remainingEdges;
        });
        return remainingNodes;
      });
    },
    [setNodes, setEdges, emitChange],
  );

  // ── Add node handler ──
  const handleAddNode = useCallback(
    (nodeId: string, command: string, args: string[]) => {
      setNodes(currentNodes => {
        const finalId = generateUniqueNodeId(nodeId, new Set(currentNodes.map(n => n.id)));
        const newNode: Node = {
          id: finalId,
          type: 'pipelineStep',
          position: { x: 400 + Math.random() * 200, y: Math.random() * 300 },
          data: { id: finalId, command, args, deps: [] } satisfies PipelineNodeData,
        };
        const updatedNodes = [...currentNodes, newNode];
        setEdges(currentEdges => {
          emitChange(updatedNodes, currentEdges);
          return currentEdges;
        });
        return updatedNodes;
      });
    },
    [setNodes, setEdges, emitChange],
  );

  // ── Edit node handler ──
  const handleEditNode = useCallback(
    (originalId: string, newNodeId: string, command: string, args: string[]) => {
      setNodes(currentNodes => {
        const finalId = generateUniqueNodeId(newNodeId, new Set(currentNodes.map(n => n.id)), originalId);
        const updatedNodes = currentNodes.map(node => {
          if (node.id === originalId) {
            return { ...node, id: finalId, data: { id: finalId, command, args, deps: [] } satisfies PipelineNodeData };
          }
          const data = node.data as PipelineNodeData;
          if (data.deps?.includes(originalId)) {
            return { ...node, data: { ...data, deps: data.deps.map(d => (d === originalId ? finalId : d)) } };
          }
          return node;
        });

        setEdges(currentEdges => {
          const updatedEdges = originalId !== finalId
            ? currentEdges.map(edge => ({
                ...edge,
                id: edge.source === originalId || edge.target === originalId
                  ? `${edge.source === originalId ? finalId : edge.source}->${edge.target === originalId ? finalId : edge.target}`
                  : edge.id,
                source: edge.source === originalId ? finalId : edge.source,
                target: edge.target === originalId ? finalId : edge.target,
              }))
            : currentEdges;
          emitChange(updatedNodes, updatedEdges);
          return updatedEdges;
        });

        return updatedNodes;
      });
    },
    [setNodes, setEdges, emitChange],
  );

  return {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    handleConnect,
    handleEdgesDelete,
    handleNodesDelete,
    handleAddNode,
    handleEditNode,
  };
}

