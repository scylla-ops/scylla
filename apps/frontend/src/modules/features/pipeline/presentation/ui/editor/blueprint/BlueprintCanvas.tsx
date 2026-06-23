import ReactFlow, { Background, BackgroundVariant, Controls, MarkerType } from 'reactflow';
import 'reactflow/dist/style.css';
import { forwardRef, useCallback, useImperativeHandle } from 'react';
import type { Node } from 'reactflow';
import { PipelineStepNode } from './PipelineStepNode.tsx';
import { StartNode } from './StartNode.tsx';
import { DeletableEdge } from './DeletableEdge.tsx';
import {
  EDGE_COLOR,
  DEFAULT_EDGE_STYLE,
  type PipelineNodeData,
  START_NODE_ID,
} from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';
import type { PipelineStep } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import { useBlueprintState } from '@/modules/features/pipeline/presentation/hooks/use-blueprint-state.ts';
import type { NodeFormValue } from './StepNodeFormDialog.tsx';

const nodeTypes = { pipelineStep: PipelineStepNode, startNode: StartNode };
const edgeTypes = { deletable: DeletableEdge };

const defaultEdgeOptions = {
  ...DEFAULT_EDGE_STYLE,
  markerEnd: { type: MarkerType.ArrowClosed, color: EDGE_COLOR },
};

export interface BlueprintCanvasHandle {
  addNode: (nodeId: string, value: NodeFormValue) => void;
  editNode: (originalId: string, newNodeId: string, value: NodeFormValue) => void;
}

interface BlueprintCanvasProps {
  steps: PipelineStep[];
  pipelineName: string;
  onStepsChange: (steps: PipelineStep[]) => void;
  onStartNodeDoubleClick: () => void;
  onStepNodeDoubleClick: (data: PipelineNodeData) => void;
}

export const BlueprintCanvas = forwardRef<BlueprintCanvasHandle, BlueprintCanvasProps>(
  ({ steps, pipelineName, onStepsChange, onStartNodeDoubleClick, onStepNodeDoubleClick }, ref) => {
    const {
      nodes,
      edges,
      onNodesChange,
      onEdgesChange,
      handleConnect,
      handleEdgesDelete,
      handleNodesDelete,
      handleAddNode,
      handleEditNode,
    } = useBlueprintState({ steps, pipelineName, onStepsChange });

    useImperativeHandle(
      ref,
      () => ({
        addNode: handleAddNode,
        editNode: handleEditNode,
      }),
      [handleAddNode, handleEditNode],
    );

    const handleNodeDoubleClick = useCallback(
      (_event: React.MouseEvent, node: Node) => {
        if (node.id === START_NODE_ID) {
          onStartNodeDoubleClick();
          return;
        }
        onStepNodeDoubleClick(node.data as PipelineNodeData);
      },
      [onStartNodeDoubleClick, onStepNodeDoubleClick],
    );

    return (
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={handleConnect}
        onEdgesDelete={handleEdgesDelete}
        onNodesDelete={handleNodesDelete}
        onNodeDoubleClick={handleNodeDoubleClick}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        deleteKeyCode={['Backspace', 'Delete']}
        edgesFocusable
        edgesUpdatable
        className='rounded-lg'
        defaultEdgeOptions={defaultEdgeOptions}
      >
        <Background variant={BackgroundVariant.Dots} gap={16} size={1} className='bg-background!' />
        <Controls className='bg-card! border-border! shadow-sm!' />
      </ReactFlow>
    );
  },
);

BlueprintCanvas.displayName = 'BlueprintCanvas';
