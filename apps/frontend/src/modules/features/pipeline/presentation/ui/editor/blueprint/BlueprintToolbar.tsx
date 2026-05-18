import { Button } from '@shadcn';
import { Plus } from 'lucide-react';
import { Trans } from '@lingui/react/macro';

interface BlueprintToolbarProps {
  onAddNode: () => void;
}

export function BlueprintToolbar({ onAddNode }: BlueprintToolbarProps) {
  return (
    <div className='absolute top-3 right-3 z-10 flex gap-2'>
      <Button size='sm' variant='outline' onClick={onAddNode}>
        <Plus className='w-4 h-4 mr-1' />
        <Trans>Add Node</Trans>
      </Button>
    </div>
  );
}

