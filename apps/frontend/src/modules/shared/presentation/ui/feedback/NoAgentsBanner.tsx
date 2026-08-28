import type { ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { useNavigate } from 'react-router-dom';
import { useAgents } from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';

interface NoAgentsBannerProps {
  /** Only warn when something is actually stuck behind the missing agent. */
  hasPendingJobs: boolean;
}

const Banner = ({ children }: { children: ReactNode }) => (
  <div className='flex items-center gap-2.5 rounded-md border px-3 py-2 text-xs text-muted-foreground'>
    <span className='h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-warning' />
    {children}
  </div>
);

/**
 * Quiet inline banner shown when jobs are queued but no agent of the org is
 * connected — without it a pending job is just a spinner that never moves.
 * Disappears on its own once an agent comes online (the agents query
 * refetches every 10s).
 *
 * Without LIST_AGENTS the agent list is never fetched, so connectivity is
 * unknowable from here: the banner then only points at agents as the likely
 * cause instead of asserting none is connected.
 */
export const NoAgentsBanner = ({ hasPendingJobs }: NoAgentsBannerProps) => {
  const { agents, isLoading, canListAgents } = useAgents();
  const navigate = useNavigate();
  const orgName = useContextStore(state => state.organization.name);

  if (!hasPendingJobs || isLoading) return null;

  if (!canListAgents) {
    return (
      <Banner>
        <span>
          <Trans>Jobs are queued — check that your agents are connected.</Trans>
        </span>
      </Banner>
    );
  }

  if (agents.some(a => a.connected)) return null;

  return (
    <Banner>
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
    </Banner>
  );
};

export default NoAgentsBanner;
