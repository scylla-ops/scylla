import { Handle, Position, useReactFlow } from 'reactflow';
import type { NodeProps } from 'reactflow';
import { Card } from '@shadcn';
import type { PipelineNodeData } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';
import { Terminal, X } from 'lucide-react';
import { useLingui } from '@lingui/react/macro';

export const PipelineStepNode = ({ id, data, selected }: NodeProps<PipelineNodeData>) => {
  const { t } = useLingui();
  const { deleteElements } = useReactFlow();

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    deleteElements({ nodes: [{ id }] });
  };

  const scriptPreview =
    data.kind === 'script' ? (data.script.split('\n').find(line => line.trim() !== '') ?? '') : '';

  return (
    <Card
      className={`w-[260px] p-0 overflow-hidden cursor-pointer transition-all duration-200 hover:scale-105 hover:border-primary hover:shadow-lg ${selected ? 'ring-2 ring-primary shadow-lg scale-105' : 'shadow-sm'}`}
    >
      <Handle
        type='target'
        position={Position.Left}
        className='w-3! h-3! bg-primary! border-2! border-background!'
      />

      <div className='bg-primary/10 px-3 py-2 flex items-center justify-between border-b'>
        <div className='flex items-center gap-2'>
          <Terminal className='w-4 h-4 text-primary' />
          <span className='font-semibold text-sm truncate'>{data.id}</span>
        </div>
        <button
          onClick={handleDelete}
          className='p-0.5 rounded hover:bg-destructive/20 text-muted-foreground hover:text-destructive transition-colors'
          title={t`Delete node`}
        >
          <X className='w-3.5 h-3.5' />
        </button>
      </div>

      <div className='px-3 py-2 space-y-1'>
        {data.kind === 'exec' ? (
          <>
            <div className='flex items-center gap-1.5'>
              <span className='text-xs text-muted-foreground'>cmd:</span>
              <code className='text-xs font-mono bg-muted px-1.5 py-0.5 rounded'>
                {data.command}
              </code>
            </div>
            {data.args.length > 0 && (
              <div className='flex items-center gap-1.5'>
                <span className='text-xs text-muted-foreground'>args:</span>
                <code className='text-xs font-mono bg-muted px-1.5 py-0.5 rounded truncate max-w-[180px]'>
                  {data.args.join(' ')}
                </code>
              </div>
            )}
          </>
        ) : (
          <div className='flex items-center gap-1.5'>
            <span className='text-[10px] font-semibold uppercase tracking-wide bg-primary/15 text-primary px-1.5 py-0.5 rounded'>
              {data.shell}
            </span>
            <code className='text-xs font-mono bg-muted px-1.5 py-0.5 rounded truncate max-w-[180px]'>
              {scriptPreview}
            </code>
          </div>
        )}
      </div>

      <Handle
        type='source'
        position={Position.Right}
        className='w-3! h-3! bg-primary! border-2! border-background!'
      />
    </Card>
  );
};
