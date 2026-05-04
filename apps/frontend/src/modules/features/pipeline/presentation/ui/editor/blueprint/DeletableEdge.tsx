import {
  BaseEdge,
  EdgeLabelRenderer,
  getSmoothStepPath,
  useReactFlow,
  type EdgeProps,
} from 'reactflow';
import { Trash2 } from 'lucide-react';

const EDGE_COLOR = 'oklch(65.752% 0.25 180)';
const SELECTED_COLOR = 'oklch(70% 0.3 30)';

export function DeletableEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  selected,
  markerEnd,
}: EdgeProps) {
  const { deleteElements } = useReactFlow();
  const [edgePath, labelX, labelY] = getSmoothStepPath({
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
  });

  const color = selected ? SELECTED_COLOR : EDGE_COLOR;

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        markerEnd={markerEnd}
        style={{
          stroke: color,
          strokeWidth: selected ? 3 : 2,
          filter: selected ? `drop-shadow(0 0 4px ${SELECTED_COLOR})` : undefined,
        }}
      />
      {selected && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              left: labelX,
              top: labelY,
              transform: 'translate(-50%, -50%)',
              pointerEvents: 'all',
              zIndex: 1000,
            }}
            className='nodrag nopan'
          >
            <button
              className='flex w-full h-full p-1 items-center justify-center rounded-full bg-destructive text-white shadow-lg cursor-pointer hover:scale-125 transition-transform border-2 border-background'
              onClick={e => {
                e.stopPropagation();
                e.preventDefault();
                console.log('Delete edge:', id);
                deleteElements({ edges: [{ id }] });
              }}
              onMouseDown={e => {
                e.stopPropagation();
              }}
              title='Delete edge'
            >
              <Trash2 className='w-6.5 h-6.5' />
            </button>
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}
