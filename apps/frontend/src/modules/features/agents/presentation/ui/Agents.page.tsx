import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAgents } from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import { createAgentItems } from '@/modules/features/agents/presentation/utils/create-agent-form-items.ts';
import { AgentCard } from '@/modules/features/agents/presentation/ui/components/AgentCard.tsx';
import { mockCardStats } from '@/modules/features/agents/presentation/utils/agent-mock-data.ts';
import type { CreatedAgent } from '@/modules/features/agents/domain/models/agent.model.ts';
import { FeatureHeader, FormDialog, SecretRevealDialog } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { Button, Card } from '@shadcn';
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
import type { FormChange } from '@shared/presentation/models/scylla-form.model.ts';
import { Cpu, Plus } from 'lucide-react';
import { Trans } from '@lingui/react/macro';

export const AgentsPage = () => {
  const { agents, isLoading, isError, createAgent, deleteAgent } = useAgents();
  const navigate = useNavigate();
  const [createOpen, setCreateOpen] = useState(false);
  const [created, setCreated] = useState<CreatedAgent | null>(null);
  const [toDelete, setToDelete] = useState<string | null>(null);

  const handleCreate = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    if (!name?.trim()) return;
    createAgent.mutate(name.trim(), {
      onSuccess: data => {
        setCreateOpen(false);
        setCreated(data);
      },
    });
  };

  const onlineCount = agents.filter(a => a.connected).length;
  const runningTotal = agents.reduce((n, a) => n + mockCardStats(a.id, a.connected).running, 0);
  const completedTotal = agents.reduce((n, a) => n + mockCardStats(a.id, a.connected).completed, 0);

  if (isError) return <ErrorState message='Error loading agents' />;

  return (
    <div className='flex flex-col gap-4 w-full h-full overflow-hidden'>
      <FeatureHeader
        count={agents.length}
        label='Agent'
        onNew={() => setCreateOpen(true)}
        underLabel={
          <>
            {!isLoading && agents.length > 0 && (
              <p className='mt-1 flex items-center gap-3 font-mono text-xs text-muted-foreground'>
                <span className='flex items-center gap-1.5'>
                  <span className='h-2 w-2 rounded-full bg-success' />
                  {onlineCount} <Trans>online</Trans>
                </span>
                <span className='flex items-center gap-1.5'>
                  <span className='h-2 w-2 rounded-full bg-destructive' />
                  {agents.length - onlineCount} <Trans>offline</Trans>
                </span>
                <span className='text-muted-foreground/70'>
                  · {runningTotal} <Trans>running</Trans> · {completedTotal}{' '}
                  <Trans>completed total</Trans>
                </span>
              </p>
            )}
          </>
        }
      />

      {isLoading ? (
        <div className='grid gap-4 sm:grid-cols-2 lg:grid-cols-3'>
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className='h-40 w-full rounded-xl' />
          ))}
        </div>
      ) : agents.length === 0 ? (
        <div className='flex flex-1 items-center justify-center'>
          <Card className='flex max-w-sm flex-col items-center gap-3 p-8 text-center'>
            <span className='relative flex h-14 w-14 items-center justify-center rounded-xl border border-dashed border-muted-foreground/40 bg-muted/30'>
              <Cpu className='h-6 w-6 text-muted-foreground' />
              <span className='absolute -bottom-1 -right-1 h-3 w-3 rounded-full border-2 border-card bg-muted-foreground/50' />
            </span>
            <h2 className='text-lg font-semibold'>
              <Trans>No agents connected</Trans>
            </h2>
            <p className='text-sm text-muted-foreground'>
              <Trans>
                An agent runs jobs. Create one to get credentials, then start the agent — it'll pull
                jobs and report back.
              </Trans>
            </p>
            <Button onClick={() => setCreateOpen(true)}>
              <Trans>Create your first agent</Trans>
            </Button>
          </Card>
        </div>
      ) : (
        <div className='grid gap-3 p-2 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3'>
          {agents.map(agent => (
            <AgentCard key={agent.id} agent={agent} onRequestDelete={setToDelete} />
          ))}
          <button
            type='button'
            onClick={() => setCreateOpen(true)}
            className='flex min-h-40 items-center justify-center gap-2 rounded-xl border border-dashed text-sm text-muted-foreground transition-colors hover:bg-accent/50 hover:text-foreground'
          >
            <Plus className='h-4 w-4' />
            <Trans>New Agent</Trans>
          </button>
        </div>
      )}

      <FormDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        title={<Trans>New Agent</Trans>}
        description={<Trans>Get credentials to connect an agent.</Trans>}
        items={createAgentItems()}
        isPending={createAgent.isPending}
        submitLabel={<Trans>Create &amp; reveal secret →</Trans>}
        onSubmit={handleCreate}
      />

      {created && (
        <SecretRevealDialog
          open={!!created}
          entityKind='agent'
          entity={{ id: created.agent.id, name: created.agent.name }}
          secret={created.secret}
          onClose={() => {
            const id = created.agent.id;
            setCreated(null);
            navigate(id);
          }}
        />
      )}

      <AlertDialog open={!!toDelete} onOpenChange={o => !o && setToDelete(null)}>
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
            <AlertDialogCancel>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (toDelete) deleteAgent.mutate(toDelete);
                setToDelete(null);
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
