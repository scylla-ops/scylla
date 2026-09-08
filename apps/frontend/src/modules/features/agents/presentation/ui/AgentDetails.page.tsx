import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  useAgent,
  useAgents,
  useAgentStats,
} from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import { AgentIdLink } from '@/modules/features/agents/presentation/ui/components/AgentIdLink.tsx';
import { OutcomesChart } from '@/modules/features/agents/presentation/ui/components/OutcomesChart.tsx';
import { AgentRunInstructions } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
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
import { Trans, useLingui } from '@lingui/react/macro';

const StripLabel = ({ children }: { children: React.ReactNode }) => (
  <span className='font-mono text-[10px] uppercase tracking-wide text-muted-foreground'>
    {children}
  </span>
);

export const AgentDetailsPage = () => {
  const { t } = useLingui();
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
    notFoundMessage: t`Agent not found`,
  });

  const online = agent?.connected ?? false;

  if (redirecting) return null;
  if (isLoading) return <Skeleton className='m-4 h-72 rounded-xl' />;
  if (isError || !agent) return <ErrorState message={<Trans>Error loading agent</Trans>} />;

  const seenLabel = agent.lastSeen ? getRelativeTime(agent.lastSeen) : null;

  return (
    <div className='w-full min-h-full flex flex-col gap-6 pb-8'>
      {/* Header */}
      <div className='flex items-center justify-between w-full gap-4'>
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
            <h1 className='text-xl font-semibold text-foreground'>{agent.name}</h1>
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

        <Button variant='destructive' onClick={() => setConfirmDelete(true)}>
          <Trans>Delete</Trans>
        </Button>
      </div>

      {/* Identity strip */}
      <div className='flex flex-wrap items-center gap-x-3 gap-y-1.5 rounded-md border border-border bg-card px-3.5 py-2 w-full'>
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
            <Trans context='date-prefix'>Created</Trans>
          </StripLabel>
          <span className='font-mono text-xs text-foreground'>{formatDate(agent.createdAt)}</span>
        </span>
        <span className='text-muted-foreground/50'>·</span>
        <span className='flex items-center gap-1.5'>
          <StripLabel>
            <Trans context='date-prefix'>Updated</Trans>
          </StripLabel>
          <span className='font-mono text-xs text-foreground'>{formatDate(agent.updatedAt)}</span>
        </span>
      </div>

      {/* Job stats */}
      <div className='w-full'>
        <div className='mb-2 flex items-baseline gap-2'>
          <h2 className='text-lg font-semibold text-foreground'>
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

        <div className='flex w-full  gap-3'>
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
      <div className='w-full'>
        <div className='mb-2 flex items-baseline gap-2'>
          <h2 className='text-lg font-semibold text-foreground'>
            <Trans>Run this agent</Trans>
          </h2>
          <span className='font-mono text-xs text-muted-foreground'>
            <Trans>connects as this agent's app id</Trans>
          </span>
        </div>
        <div className='w-full'>
          <AgentRunInstructions appId={agent.id} />
        </div>
      </div>

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
