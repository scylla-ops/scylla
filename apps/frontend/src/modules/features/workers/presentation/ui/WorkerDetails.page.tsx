import { useParams } from 'react-router-dom';
import {
  useWorker,
  useWorkerStats,
} from '@/modules/features/workers/presentation/hooks/use-workers.ts';
import { BackButton } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { Badge } from '@shadcn';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Trans } from '@lingui/react/macro';

export const WorkerDetailsPage = () => {
  const { workerId } = useParams<{ workerId: string }>();
  const { data: worker, isLoading, isError } = useWorker(workerId ?? '');
  const { data: stats } = useWorkerStats(workerId ?? '');

  if (isLoading) return <></>;
  if (isError || !worker) return <ErrorState message='Error loading worker' />;

  return (
    <div className='space-y-4 p-4'>
      <BackButton />
      <div className='flex items-center gap-3'>
        <h1 className='text-xl font-semibold'>{worker.name}</h1>
        <Badge variant={worker.connected ? 'default' : 'secondary'}>
          {worker.connected ? <Trans>online</Trans> : <Trans>offline</Trans>}
        </Badge>
      </div>
      <dl className='grid gap-3 text-sm sm:grid-cols-2'>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Worker ID</Trans>
          </dt>
          <dd className='font-mono break-all'>{worker.id}</dd>
        </div>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Active</Trans>
          </dt>
          <dd>{String(worker.isActive)}</dd>
        </div>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Last seen</Trans>
          </dt>
          <dd>{worker.lastSeen ? formatDate(worker.lastSeen) : '—'}</dd>
        </div>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Created</Trans>
          </dt>
          <dd>{formatDate(worker.createdAt)}</dd>
        </div>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Updated</Trans>
          </dt>
          <dd>{formatDate(worker.updatedAt)}</dd>
        </div>
      </dl>

      {stats && (
        <dl className='grid gap-3 text-sm sm:grid-cols-2'>
          <div>
            <dt className='text-muted-foreground'>
              <Trans>Total jobs</Trans>
            </dt>
            <dd>{stats.total}</dd>
          </div>
          <div>
            <dt className='text-muted-foreground'>
              <Trans>Completed</Trans>
            </dt>
            <dd>{stats.completed}</dd>
          </div>
          <div>
            <dt className='text-muted-foreground'>
              <Trans>Running</Trans>
            </dt>
            <dd>{stats.running}</dd>
          </div>
          <div>
            <dt className='text-muted-foreground'>
              <Trans>Failed</Trans>
            </dt>
            <dd>{stats.failed}</dd>
          </div>
          <div>
            <dt className='text-muted-foreground'>
              <Trans>Last run</Trans>
            </dt>
            <dd>{stats.lastRunAt ? formatDate(stats.lastRunAt) : '—'}</dd>
          </div>
        </dl>
      )}
    </div>
  );
};
