import { DialogDescription, DialogHeader, DialogTitle } from '@shadcn';
import { ShieldCheck } from 'lucide-react';
import { Trans } from '@lingui/react/macro';

interface RoleDialogHeaderProps {
  isEdit: boolean;
}

export const RoleDialogHeader = ({ isEdit }: RoleDialogHeaderProps) => {
  return (
    <DialogHeader className='space-y-3'>
      <DialogTitle className='flex items-center gap-2.5 text-lg font-semibold'>
        <div className='flex items-center justify-center size-8 rounded-lg bg-primary/10'>
          <ShieldCheck className='size-4 text-primary' />
        </div>
        <span>{isEdit ? <Trans>Edit role</Trans> : <Trans>Create role</Trans>}</span>
      </DialogTitle>
      <DialogDescription>
        <Trans>Define what this role is called and what it can do.</Trans>
      </DialogDescription>
    </DialogHeader>
  );
};
