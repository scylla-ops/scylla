import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  useAgent,
  useAgents,
  useAgentStats,
} from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import { AgentIdLink } from '@/modules/features/agents/presentation/ui/components/AgentIdLink.tsx';
import { OutcomesChart } from '@/modules/features/agents/presentation/ui/components/OutcomesChart.tsx';
import { AgentRunInstructions, BackButton } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { useResourceError } from '@shared/presentation/hooks/use-resource-error.ts';
import { Badge, Button } from '@shadcn';
import { Skeleton } from '@shadcn/skeleton.tsx';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@shadcn/alert-dialog.tsx';
import { Cpu } from 'lucide-react';
import { cn } from '@shared/presentation/utils';
import { formatDate, getRelativeTime } from '@shared/utils/date-utils.ts';
import { Trans } from '@lingui/react/macro';

const StripLabel = ({ children }: { children: React.ReactNode }) => (
  <span className='font-mono text-[10px] uppercase tracking-wide text-muted-foreground'>
    {children}
  </span>
);

export const AgentDetailsPage = () => {
  const { agentId } = useParams<{ agentId: string }>();
  const navigate = useNavigate();
  const { data: agent, isLoading, isError, error } = useAgent(agentId ?? '');
  const { data: stats } = useAgentStats(agentId ?? '');
  const { deleteAgent } = useAgents();
  const [confirmDelete, setConfirmDelete] = useState(false);

  // NOT_FOUND (deleted / bad id) → toast + back to the agents list.
  const { redirecting } = useResourceError({
    error,
    redirectTo: '..',
    notFoundMessage: 'Agent not found',
  });

  const online = agent?.connected ?? false;

  if (redirecting) return null;
  if (isLoading) return <Skeleton className='m-4 h-72 rounded-xl' />;
  if (isError || !agent) return <ErrorState message='Error loading agent' />;

  const seenLabel = agent.lastSeen ? getRelativeTime(agent.lastSeen) : null;

  return (
    <div className='w-full h-full flex flex-col gap-4'>
      {/* Header — no ULID here; it lives in the identity strip as a link. */}
      <div className={'flex flex-row gap-2'}>
        <BackButton iconOnly />
        <div className={'flex flex-row justify-between w-full '}>
          <div className='flex items-start justify-between gap-3'>
            <div className='flex items-center gap-3'>
              <span
                className={cn(
                  'relative flex h-14 w-14 items-center justify-center rounded-lg bg-success/10',
                  online ? 'border border-success' : 'border-2 border-destructive',
                )}
              >
                <Cpu className={cn('h-7 w-7', online ? 'text-success' : 'text-destructive')} />
                <span className='absolute -bottom-1 -right-1 flex h-3.5 w-3.5' aria-hidden>
                  {online && (
                    <span className='absolute inline-flex h-full w-full animate-ping rounded-full bg-success/70' />
                  )}
                  <span
                    className={cn(
                      'relative inline-flex h-3.5 w-3.5 rounded-full border-2 border-card',
                      online ? 'bg-success' : 'bg-destructive',
                    )}
                  />
                </span>
              </span>
              <div>
                <h1 className='text-xl font-semibold'>{agent.name}</h1>
                <p className='font-mono text-xs text-muted-foreground'>
                  {online ? <Trans>online</Trans> : <Trans>offline</Trans>}
                  {seenLabel && (
                    <>
                      {' · '}
                      {online ? <Trans>seen</Trans> : <Trans>down</Trans>} {seenLabel}
                    </>
                  )}
                </p>
              </div>
            </div>
          </div>
        </div>
        <div className='flex items-center gap-2'>
          <Button variant='destructive' onClick={() => setConfirmDelete(true)}>
            <Trans>Delete</Trans>
          </Button>
        </div>
      </div>

      {/* Identity strip — the quietest block on the page. */}
      <div className='flex flex-wrap items-center gap-x-3 gap-y-1 rounded-md border px-3.5 py-2'>
        <span className='flex items-center gap-1.5'>
          <StripLabel>
            <Trans>Agent ID</Trans>
          </StripLabel>
          <AgentIdLink id={agent.id} />
        </span>
        <span className='text-muted-foreground/50'>·</span>
        <span className='flex items-center gap-1.5'>
          <StripLabel>
            <Trans>Active</Trans>
          </StripLabel>
          <Badge variant={agent.isActive ? 'default' : 'secondary'}>
            {agent.isActive ? <Trans>active</Trans> : <Trans>inactive</Trans>}
          </Badge>
        </span>
        <span className='text-muted-foreground/50'>·</span>
        <span className='flex items-center gap-1.5'>
          <StripLabel>
            <Trans>Created</Trans>
          </StripLabel>
          <span className='font-mono text-xs'>{formatDate(agent.createdAt)}</span>
        </span>
        <span className='text-muted-foreground/50'>·</span>
        <span className='flex items-center gap-1.5'>
          <StripLabel>
            <Trans>Updated</Trans>
          </StripLabel>
          <span className='font-mono text-xs'>{formatDate(agent.updatedAt)}</span>
        </span>
      </div>

      {/* Job stats */}
      <div>
        <div className='mb-2 flex items-baseline gap-2'>
          <h2 className='text-lg font-semibold'>
            <Trans>Job stats</Trans>
          </h2>
          {stats && (
            <span className='font-mono text-xs text-muted-foreground'>
              <Trans>since</Trans> {formatDate(agent.createdAt)}
              {stats.lastRunAt && (
                <>
                  {' · '}
                  <Trans>last run</Trans> {getRelativeTime(stats.lastRunAt)}
                </>
              )}
            </span>
          )}
        </div>

        <div className='grid gap-3 lg:grid-cols-[380px_1fr]'>
          {stats ? (
            <OutcomesChart
              daily={stats.daily}
              aggregate={{
                completed: stats.completed,
                failed: stats.failed,
                cancelled: stats.cancelled,
              }}
            />
          ) : (
            <Skeleton className='h-64 w-full rounded-xl' />
          )}
        </div>
      </div>

      {/* How to start a worker for this agent */}
      <div>
        <div className='mb-2 flex items-baseline gap-2'>
          <h2 className='text-lg font-semibold'>
            <Trans>Run this worker</Trans>
          </h2>
          <span className='font-mono text-xs text-muted-foreground'>
            <Trans>connects as this agent's app id</Trans>
          </span>
        </div>
        <div className='max-w-3xl'>
          <AgentRunInstructions appId={agent.id} />
        </div>
      </div>

      {/* Logs  here (not functionnal yet, need plug to backend */}

      <AlertDialog open={confirmDelete} onOpenChange={setConfirmDelete}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Delete agent?</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              <Trans>This revokes the agent's grants and disconnects it. Cannot be undone.</Trans>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setConfirmDelete(false)}>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                deleteAgent.mutate(agent.id, {
                  onSuccess: () => {
                    void navigate('..');
                  },
                });
                setConfirmDelete(false);
              }}
            >
              <Trans>Delete</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
