import { Handle, Position } from 'reactflow';
import type { NodeProps } from 'reactflow';
import { Card } from '@shadcn';
import { Play } from 'lucide-react';
import { Trans } from '@lingui/react/macro';

interface StartNodeData {
  name: string;
}

interface StartNodeProps extends NodeProps<StartNodeData> {
  onNameChange?: (name: string) => void;
}

export const StartNode = ({ data }: StartNodeProps) => {
  return (
    <Card className='w-55 p-0 overflow-hidden border-primary/30 cursor-pointer transition-all duration-200 hover:scale-105 hover:border-primary hover:shadow-lg shadow-md'>
      <div className='bg-primary px-3 py-2 flex items-center gap-2'>
        <Play className='w-4 h-4 text-primary-foreground fill-primary-foreground' />
        <span className='font-bold text-sm text-primary-foreground'>
          <Trans>Pipeline</Trans>
        </span>
      </div>

      <div className='px-3 py-3'>
        <span className='text-sm font-medium truncate block'>{data.name || 'Unnamed'}</span>
      </div>

      <Handle
        type='source'
        position={Position.Right}
        className='w-3! h-3! bg-primary! border-2! border-background!'
      />
    </Card>
  );
};




