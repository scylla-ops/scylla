import { useMemo } from 'react';
import { Trans } from '@lingui/react/macro';
import { Button } from '@shadcn';
import { ExternalLink } from 'lucide-react';
import {
  mockInitialLogs,
  type LogLevel,
} from '@/modules/features/agents/presentation/utils/agent-mock-data.ts';

interface AgentLogsProps {
  agentId: string;
  agentName: string;
}

const LEVEL_COLOR: Record<LogLevel, string> = {
  info: '#9aa0a6',
  ok: '#7adfa1',
  warn: '#f0c460',
  error: '#f08581',
};

// Static preview only. The agent log stream does not exist yet; this renders a
// fixed sample so the layout reads correctly. Wire the real stream when ready.
export const AgentLogs = ({ agentId, agentName }: AgentLogsProps) => {
  const lines = useMemo(() => mockInitialLogs(agentId), [agentId]);

  return (
    <div className='space-y-2'>
      <div className='flex items-center justify-between gap-2'>
        <div className='flex items-baseline gap-2'>
          <h2 className='text-lg font-semibold'>
            <Trans>Logs</Trans>
          </h2>
          <span className='font-mono text-xs text-muted-foreground'>
            <Trans>preview · agent log stream not wired yet</Trans>
          </span>
        </div>
        <div className='flex items-center gap-2'>
          <span className='rounded-full border bg-muted px-2 py-0.5 text-[11px] font-mono text-muted-foreground'>
            <Trans>sample</Trans>
          </span>
          <Button variant='outline' size='sm' disabled title='Coming soon'>
            <Trans>filter</Trans> ▾
          </Button>
          <Button variant='outline' size='sm' disabled title='Coming soon'>
            <Trans>open full logs</Trans> <ExternalLink className='ml-1 h-3 w-3' />
          </Button>
        </div>
      </div>

      <div className='overflow-hidden rounded-md' style={{ background: '#1a1816' }}>
        <div
          className='flex items-center justify-between px-3 py-1.5 font-mono text-[11px]'
          style={{ borderBottom: '1px solid rgba(255,255,255,0.08)', color: '#9aa0a6' }}
        >
          <span>~/agent/{agentName}.log</span>
          <span>
            {lines.length} <Trans>lines</Trans>
          </span>
        </div>
        <div className='max-h-[240px] overflow-y-auto px-3 py-2'>
          {lines.map((l, i) => (
            <div
              key={i}
              className='grid items-baseline gap-2.5 py-px font-mono text-[11px]'
              style={{ gridTemplateColumns: '76px 50px 1fr' }}
            >
              <span style={{ color: '#666' }}>{l.t}</span>
              <span style={{ color: LEVEL_COLOR[l.level] }}>{l.level.toUpperCase()}</span>
              <span className='truncate' style={{ color: '#e8e6e2', whiteSpace: 'nowrap' }}>
                {l.msg}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
