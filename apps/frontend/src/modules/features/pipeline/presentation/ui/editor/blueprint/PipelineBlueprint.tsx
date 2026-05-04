import ReactFlow, {
  addEdge,
  Background,
  BackgroundVariant,
  type Connection,
  Controls,
  type Edge,
  MarkerType,
  type Node,
  useEdgesState,
  useNodesState,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { useCallback, useEffect, useRef, useState } from 'react';
import { Button } from '@shadcn';
import { Plus } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { PipelineStepNode } from './PipelineStepNode.tsx';
import { StartNode } from './StartNode.tsx';
import { AddNodeDialog } from './AddNodeDialog.tsx';
import { EditNodeDialog } from './EditNodeDialog.tsx';
import { EditStartNodeDialog } from './EditStartNodeDialog.tsx';
import { DeletableEdge } from './DeletableEdge.tsx';
import {
  flowToSteps,
  type PipelineNodeData,
  START_NODE_ID,
  stepsToFlow,
} from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

const EDGE_COLOR = 'oklch(65.752% 0.25 180)';

function uniqueNodeId(desired: string, existingIds: Set<string>, excludeId?: string): string {
  const ids = new Set(existingIds);
  if (excludeId) ids.delete(excludeId);
  if (!ids.has(desired)) return desired;
  let i = 1;
  while (ids.has(`${desired}_${i}`)) i++;
  return `${desired}_${i}`;
}

interface PipelineBlueprintProps {
  steps: PipelineStep[];
  pipelineName: string;
  onStepsChange: (steps: PipelineStep[]) => void;
  onNameChange: (name: string) => void;
}

const nodeTypes = {
  pipelineStep: PipelineStepNode,
  startNode: StartNode,
};

const edgeTypes = {
  deletable: DeletableEdge,
};

export const PipelineBlueprint = ({
  steps,
  pipelineName,
  onStepsChange,
  onNameChange,
}: PipelineBlueprintProps) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [editStartDialogOpen, setEditStartDialogOpen] = useState(false);
  const [editingNode, setEditingNode] = useState<PipelineNodeData | null>(null);
  const isSyncingFromBlueprint = useRef(false);

  // Sync from parent (script) → blueprint
  useEffect(() => {
    if (isSyncingFromBlueprint.current) {
      isSyncingFromBlueprint.current = false;
      return;
    }
    const { nodes: newNodes, edges: newEdges, sanitizedSteps } = stepsToFlow(steps, pipelineName);
    setNodes(newNodes);
    setEdges(newEdges);

    // Propagate sanitized steps back to parent if they differ
    if (JSON.stringify(sanitizedSteps) !== JSON.stringify(steps)) {
      isSyncingFromBlueprint.current = true;
      onStepsChange(sanitizedSteps);
    }
  }, [steps, pipelineName, setNodes, setEdges, onStepsChange]);

  // Sync blueprint → parent (script), skipping the next parent→blueprint sync
  const syncToParent = useCallback(
    (currentNodes: Node[], currentEdges: Edge[]) => {
      isSyncingFromBlueprint.current = true;
      onStepsChange(flowToSteps(currentNodes, currentEdges));
    },
    [onStepsChange],
  );

  const onConnect = useCallback(
    (connection: Connection) => {
      // Don't allow connecting TO the start node
      if (connection.target === START_NODE_ID) return;

      setEdges(eds => {
        const newEdges = addEdge(
          {
            ...connection,
            animated: true,
            type: 'deletable',
            markerEnd: { type: MarkerType.ArrowClosed, color: EDGE_COLOR },
            style: { stroke: EDGE_COLOR, strokeWidth: 2 },
          },
          eds,
        );
        setNodes(nds => {
          syncToParent(nds, newEdges);
          return nds;
        });
        return newEdges;
      });
    },
    [setEdges, setNodes, syncToParent],
  );

  const onEdgesDelete = useCallback(
    (deletedEdges: Edge[]) => {
      setEdges(eds => {
        const deletedIds = new Set(deletedEdges.map(e => e.id));
        const remaining = eds.filter(e => !deletedIds.has(e.id));
        setNodes(nds => {
          syncToParent(nds, remaining);
          return nds;
        });
        return remaining;
      });
    },
    [setEdges, setNodes, syncToParent],
  );

  const onNodesDelete = useCallback(
    (deletedNodes: Node[]) => {
      const deletedIds = new Set(deletedNodes.map(n => n.id));
      setNodes(nds => {
        const remainingNodes = nds.filter(n => !deletedIds.has(n.id));
        setEdges(eds => {
          const remainingEdges = eds.filter(
            e => !deletedIds.has(e.source) && !deletedIds.has(e.target),
          );
          syncToParent(remainingNodes, remainingEdges);
          return remainingEdges;
        });
        return remainingNodes;
      });
    },
    [setNodes, setEdges, syncToParent],
  );

  const handleAddNode = useCallback(
    (nodeId: string, command: string, args: string[]) => {
      setNodes(nds => {
        const existingIds = new Set(nds.map(n => n.id));
        const finalId = uniqueNodeId(nodeId, existingIds);
        const newNode: Node = {
          id: finalId,
          type: 'pipelineStep',
          position: { x: 400 + Math.random() * 200, y: Math.random() * 300 },
          data: { id: finalId, command, args, deps: [] },
        };
        const updated = [...nds, newNode];
        setEdges(eds => {
          syncToParent(updated, eds);
          return eds;
        });
        return updated;
      });
    },
    [setNodes, setEdges, syncToParent],
  );

  const onNodeDoubleClick = useCallback((_event: React.MouseEvent, node: Node) => {
    if (node.id === START_NODE_ID) {
      setEditStartDialogOpen(true);
      return;
    }
    setEditingNode(node.data as PipelineNodeData);
    setEditDialogOpen(true);
  }, []);

  const handleEditNode = useCallback(
    (originalId: string, newNodeId: string, command: string, args: string[]) => {
      setNodes(nds => {
        const existingIds = new Set(nds.map(n => n.id));
        const finalId = uniqueNodeId(newNodeId, existingIds, originalId);
        const updated = nds.map(n => {
          if (n.id === originalId) {
            return {
              ...n,
              id: finalId,
              data: { id: finalId, command, args, deps: [] },
            };
          }
          // Update deps references in other nodes
          const data = n.data as PipelineNodeData;
          if (data.deps?.includes(originalId)) {
            return {
              ...n,
              data: {
                ...data,
                deps: data.deps.map(d => (d === originalId ? finalId : d)),
              },
            };
          }
          return n;
        });
        setEdges(eds => {
          const updatedEdges =
            originalId !== finalId
              ? eds.map(e => ({
                  ...e,
                  id: e.source === originalId || e.target === originalId
                    ? `${e.source === originalId ? finalId : e.source}->${e.target === originalId ? finalId : e.target}`
                    : e.id,
                  source: e.source === originalId ? finalId : e.source,
                  target: e.target === originalId ? finalId : e.target,
                }))
              : eds;
          syncToParent(updated, updatedEdges);
          return updatedEdges;
        });
        return updated;
      });
    },
    [setNodes, setEdges, syncToParent],
  );

  return (
    <div className='h-full w-full relative'>
      <div className='absolute top-3 right-3 z-10 flex gap-2'>
        <Button size='sm' variant='outline' onClick={() => setAddDialogOpen(true)}>
          <Plus className='w-4 h-4 mr-1' />
          <Trans>Add Node</Trans>
        </Button>
      </div>

      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onEdgesDelete={onEdgesDelete}
        onNodesDelete={onNodesDelete}
        onNodeDoubleClick={onNodeDoubleClick}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        deleteKeyCode={['Backspace', 'Delete']}
        edgesFocusable
        edgesUpdatable
        className='rounded-lg'
        defaultEdgeOptions={{
          type: 'deletable',
          animated: true,
          style: { stroke: EDGE_COLOR, strokeWidth: 2 },
          markerEnd: { type: MarkerType.ArrowClosed, color: EDGE_COLOR },
        }}
      >
        <Background variant={BackgroundVariant.Dots} gap={16} size={1} className='bg-background!' />
        <Controls className='bg-card! border-border! shadow-sm!' />
      </ReactFlow>

      <AddNodeDialog open={addDialogOpen} setOpen={setAddDialogOpen} onAdd={handleAddNode} />
      <EditNodeDialog
        open={editDialogOpen}
        setOpen={setEditDialogOpen}
        nodeData={editingNode}
        onSave={handleEditNode}
      />
      <EditStartNodeDialog
        open={editStartDialogOpen}
        setOpen={setEditStartDialogOpen}
        name={pipelineName}
        onSave={onNameChange}
      />
    </div>
  );
};
