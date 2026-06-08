import { useSecrets } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import {
  SecretHeader,
  SecretList,
} from '@/modules/features/secret/presentation/ui/components/index.ts';
import { useParams } from 'react-router-dom';
import { CreateSecretDialog } from '@/modules/features/secret/presentation/ui/CreateSecretDialog.tsx';
import { useState } from 'react';

export const SecretPage = () => {
  const { projectId } = useParams();

  const { secrets } = useSecrets(projectId!);

  const [isCreateDialogOpen, setCreateIsDialogOpen] = useState(false);

  if (projectId == undefined) return <></>;

  return (
    <div className='flex flex-col gap-4 w-full min-h-full'>
      <SecretHeader
        activeCount={secrets.length}
        onAddSecret={() => {
          setCreateIsDialogOpen(true);
        }}
      />
      <div className='overflow-hidden'>
        <SecretList secrets={secrets} projectId={projectId} />
        <CreateSecretDialog
          projectId={projectId!}
          isOpen={isCreateDialogOpen}
          setOpen={setCreateIsDialogOpen}
        />
      </div>
    </div>
  );
};

export default SecretPage;
