import { useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { Copy, Cpu, MoreHorizontal, Trash } from 'lucide-react';
import { Badge, Button, Card, CardContent } from '@shadcn';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import type { Agent } from '@/modules/features/agents/domain/models/agent.model.ts';
import { AgentIdLink } from '@/modules/features/agents/presentation/ui/components/AgentIdLink.tsx';
import { Sparkline } from '@/modules/features/agents/presentation/ui/components/Sparkline.tsx';
import {
  mockCardStats,
  mockSparkline,
} from '@/modules/features/agents/presentation/utils/agent-mock-data.ts';
import { cn } from '@shared/presentation/utils';
import { formatDate, getRelativeTime } from '@shared/utils/date-utils.ts';
import { toast } from 'sonner';

interface AgentCardProps {
  agent: Agent;
  onRequestDelete: (id: string) => void;
}

export const AgentCard = ({ agent, onRequestDelete }: AgentCardProps) => {
  const navigate = useNavigate();
  const cardStats = useMemo(
    () => mockCardStats(agent.id, agent.connected),
    [agent.id, agent.connected],
  );
  const sparkData = useMemo(
    () => mockSparkline(agent.id, agent.connected),
    [agent.id, agent.connected],
  );

  const copyId = async () => {
    await navigator.clipboard.writeText(agent.id);
    toast.success('Agent id copied');
  };

  return (
    <Card
      className='cursor-pointer gap-0 py-0 transition-colors hover:bg-accent/50'
      onClick={() => navigate(agent.id)}
    >
      <CardContent className='p-4'>
        <div className='flex items-start justify-between gap-2'>
          <div className='flex min-w-0 items-center gap-3'>
            <span
              className={cn(
                'relative flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-success/10',
                agent.connected ? 'border border-success' : 'border-2 border-destructive',
              )}
            >
              <Cpu
                className={cn('h-4 w-4', agent.connected ? 'text-success' : 'text-destructive')}
              />
              {/* live status dot */}
              <span className='absolute -bottom-1 -right-1 flex h-3 w-3'>
                {agent.connected && (
                  <span className='absolute inline-flex h-full w-full animate-ping rounded-full bg-success/70' />
                )}
                <span
                  className={cn(
                    'relative inline-flex h-3 w-3 rounded-full border-2 border-card',
                    agent.connected ? 'bg-success' : 'bg-destructive',
                  )}
                />
              </span>
            </span>
            <div className='min-w-0'>
              <p className='truncate font-semibold'>{agent.name}</p>
              <AgentIdLink id={agent.id} truncate={18} />
            </div>
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='ghost'
                size='icon'
                className='h-7 w-7 shrink-0'
                onClick={e => e.stopPropagation()}
              >
                <MoreHorizontal className='h-4 w-4' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end' onClick={e => e.stopPropagation()}>
              <DropdownMenuItem onClick={copyId}>
                <Copy className='mr-2 h-4 w-4' />
                <Trans>Copy id</Trans>
              </DropdownMenuItem>
              <DropdownMenuItem variant='destructive' onClick={() => onRequestDelete(agent.id)}>
                <Trash className='mr-2 h-4 w-4' />
                <Trans>Delete</Trans>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        {/* status row */}
        <div className='mt-3 flex items-center justify-between gap-2'>
          {agent.connected ? (
            <Badge className='gap-1 bg-success/15 text-success hover:bg-success/15'>
              <span className='h-1.5 w-1.5 rounded-full bg-success' />
              <Trans>online</Trans>
            </Badge>
          ) : (
            <Badge variant='secondary' className='gap-1 text-destructive'>
              <span className='h-1.5 w-1.5 rounded-full bg-destructive' />
              <Trans>offline</Trans>
            </Badge>
          )}
          <span className='font-mono text-xs text-muted-foreground'>
            {agent.lastSeen ? (
              agent.connected ? (
                <>
                  <Trans>seen</Trans> {getRelativeTime(agent.lastSeen)}
                </>
              ) : (
                <>
                  <Trans>down</Trans> {getRelativeTime(agent.lastSeen)}
                </>
              )
            ) : (
              <Trans>never connected</Trans>
            )}
          </span>
        </div>

        {/* stats strip */}
        {cardStats.hasRuns ? (
          <div className='mt-3 flex items-center justify-between gap-3 rounded-md bg-muted/50 px-3 py-2'>
            <div className='min-w-0'>
              <p className='font-mono text-[10px] uppercase tracking-wide text-muted-foreground'>
                <Trans>runs · last hour</Trans>
              </p>
              <Sparkline data={sparkData} width={150} height={20} label='runs in the last hour' />
            </div>
            <div className='text-right'>
              <p className='font-mono text-[10px] uppercase tracking-wide text-muted-foreground'>
                <Trans>completed</Trans>
              </p>
              <p className='font-mono text-sm font-semibold'>{cardStats.completed}</p>
            </div>
          </div>
        ) : (
          <div className='mt-3 rounded-md bg-muted/50 px-3 py-2 text-center text-xs text-muted-foreground'>
            <Trans>no runs yet · waiting for first job</Trans>
          </div>
        )}
      </CardContent>

      <div className='flex items-center justify-between border-t border-dashed px-4 py-2.5'>
        <span className='text-xs text-muted-foreground'>
          <Trans>created</Trans> {formatDate(agent.createdAt)}
        </span>
        <div className='flex items-center gap-2'>
          {cardStats.running > 0 && (
            <span className='font-mono text-xs font-semibold text-success'>
              {cardStats.running} <Trans>running</Trans>
            </span>
          )}
          <Button
            variant='ghost'
            size='icon'
            className='h-7 w-7 text-muted-foreground hover:text-destructive'
            onClick={e => {
              e.stopPropagation();
              onRequestDelete(agent.id);
            }}
          >
            <Trash className='h-4 w-4' />
          </Button>
        </div>
      </div>
    </Card>
  );
};
