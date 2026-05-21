import { useParams } from 'react-router-dom';
import { useApp } from '@/modules/features/apps/presentation/hooks/use-apps.ts';
import { BackButton } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Trans } from '@lingui/react/macro';

export const AppDetailsPage = () => {
  const { appId } = useParams<{ appId: string }>();
  const { data: app, isLoading, isError } = useApp(appId ?? '');

  if (isLoading) return <></>;
  if (isError || !app) return <ErrorState message='Error loading app' />;

  return (
    <div className='space-y-4 p-4'>
      <BackButton />
      <div className='flex items-center gap-3'>
        <h1 className='text-xl font-semibold'>{app.name}</h1>
      </div>
      <dl className='grid gap-3 text-sm sm:grid-cols-2'>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>App ID</Trans>
          </dt>
          <dd className='font-mono break-all'>{app.id}</dd>
        </div>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Active</Trans>
          </dt>
          <dd>{String(app.isActive)}</dd>
        </div>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Created</Trans>
          </dt>
          <dd>{formatDate(app.createdAt)}</dd>
        </div>
        <div>
          <dt className='text-muted-foreground'>
            <Trans>Updated</Trans>
          </dt>
          <dd>{formatDate(app.updatedAt)}</dd>
        </div>
      </dl>
    </div>
  );
};
