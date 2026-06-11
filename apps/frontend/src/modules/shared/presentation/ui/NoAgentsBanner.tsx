import { Trans } from '@lingui/react/macro';
import { useNavigate } from 'react-router-dom';
import { useAgents } from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';

interface NoAgentsBannerProps {
  /** Only warn when something is actually stuck behind the missing agent. */
  hasPendingJobs: boolean;
}

/**
 * Quiet inline banner shown when jobs are queued but no agent of the org is
 * connected — without it a pending job is just a spinner that never moves.
 * Disappears on its own once an agent comes online (the agents query
 * refetches every 10s).
 */
export const NoAgentsBanner = ({ hasPendingJobs }: NoAgentsBannerProps) => {
  const { agents, isLoading } = useAgents();
  const navigate = useNavigate();
  const orgName = useContextStore(state => state.organization.name);

  const anyConnected = agents.some(a => a.connected);
  if (!hasPendingJobs || isLoading || anyConnected) return null;

  return (
    <div className='flex items-center gap-2.5 rounded-md border px-3 py-2 text-xs text-muted-foreground'>
      <span className='h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-warning' />
      <span>
        <Trans>No agent connected — queued jobs are waiting for one.</Trans>
      </span>
      <button
        type='button'
        onClick={() => void navigate(`${orgName ? `/${slugifyOrgName(orgName)}` : ''}/agents`)}
        className='ml-auto shrink-0 font-medium text-primary hover:underline'
      >
        <Trans>Set up an agent</Trans> →
      </button>
    </div>
  );
};

export default NoAgentsBanner;
