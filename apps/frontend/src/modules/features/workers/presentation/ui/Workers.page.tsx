import { WorkersTable } from '@/modules/features/workers/presentation/ui/workers-table/WorkersTable.tsx';
import { useWorkers } from '@/modules/features/workers/presentation/hooks/use-workers.ts';
import { FeatureHeader } from '@shared/presentation/ui';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { Input } from '@shadcn';

export const WorkersPage = () => {
  const { workers, isLoading, isError, error, searchTerm, setSearchTerm } = useWorkers();
  const { goToWorkerDetails } = useScyllaNavigate();

  if (isLoading) return <></>;
  if (isError) {
    console.log(error);
    return <ErrorState message='Error loading workers' />;
  }

  return (
    <div className={'flex flex-col gap-4 w-full p-2'}>
      <FeatureHeader count={workers?.length ?? 0} label='Worker' onNew={undefined} />

      <div className='px-1'>
        <Input
          placeholder='Search workers by hostname, ID, or status...'
          value={searchTerm}
          onChange={e => setSearchTerm(e.target.value)}
          className='max-w-md'
        />
      </div>

      <WorkersTable onView={goToWorkerDetails} data={workers} />
    </div>
  );
};
