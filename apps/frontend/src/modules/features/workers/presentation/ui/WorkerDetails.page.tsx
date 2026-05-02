import { useParams } from 'react-router-dom';
import { useWorker } from '@/modules/features/workers/presentation/hooks/use-workers.ts';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { formatDate } from '@shared/utils/date-utils.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { BackButton } from '@shared/presentation/ui/BackButton.tsx';
import { Trans } from '@lingui/react/macro';

export const WorkerDetailsPage = () => {
  const { workerId } = useParams();
  const { data: worker, isLoading, isError } = useWorker(workerId ?? '');
  const { goBack } = useScyllaNavigate();

  if (isLoading) return <></>;
  if (isError || !worker) return <ErrorState message='Error loading worker details' />;

  return (
    <div className='flex flex-col gap-6 w-full p-4'>
      <div className='flex flex-col gap-4'>
        <div className='flex items-center justify-between gap-4'>
          <div>
            <h1 className='text-3xl font-bold tracking-tight'>
              <Trans>Worker details</Trans>
            </h1>
            <p className='text-sm text-muted-foreground'>
              <Trans>Information about the selected worker.</Trans>
            </p>
          </div>
          <BackButton variant='outline' onClick={() => goBack()} label={<Trans>Back</Trans>} />
        </div>

        <div className='grid gap-4 sm:grid-cols-2'>
          <div className='rounded-xl border border-slate-200 bg-white p-6 shadow-sm'>
            <h2 className='text-xl font-semibold mb-3'>
              <Trans>Worker</Trans>
            </h2>
            <div className='space-y-3 text-sm text-slate-700'>
              <div>
                <span className='font-semibold'>
                  <Trans>Hostname:</Trans>
                </span>{' '}
                {worker.hostname}
              </div>
              <div>
                <span className='font-semibold'>
                  <Trans>Agent ID:</Trans>
                </span>{' '}
                {worker.agentId}
              </div>
              <div>
                <span className='font-semibold'>
                  <Trans>Status:</Trans>
                </span>{' '}
                {worker.status}
              </div>
              <div>
                <span className='font-semibold'>
                  <Trans>Last seen at:</Trans>
                </span>{' '}
                {formatDate(worker.lastSeenAt)}
              </div>
              <div>
                <span className='font-semibold'>
                  <Trans>Created at:</Trans>
                </span>{' '}
                {formatDate(worker.createdAt)}
              </div>
              <div>
                <span className='font-semibold'>
                  <Trans>Updated at:</Trans>
                </span>{' '}
                {formatDate(worker.updatedAt)}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
