import { useMemo, useState } from 'react';
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from 'recharts';
import { Trans } from '@lingui/react/macro';
import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { Skeleton } from '@shadcn/skeleton.tsx';
import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@shadcn/chart.tsx';
import { cn } from '@shared/presentation/utils';
import { useAgents } from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import { useAgentStats } from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import type { DailyOutcome } from '@/modules/features/agents/domain/structs/agent.struct.ts';

type Range = '7d' | '14d' | '30d';
type StatusFilter = 'all' | 'completed' | 'failed' | 'cancelled';

const RANGES: Range[] = ['7d', '14d', '30d'];
const RANGE_DAYS: Record<Range, number> = { '7d': 7, '14d': 14, '30d': 30 };
const STATUS_FILTERS: StatusFilter[] = ['all', 'completed', 'failed', 'cancelled'];

const CHART_CONFIG = {
  completed: { label: 'Completed', color: 'var(--success)' },
  failed: { label: 'Failed', color: 'var(--destructive)' },
  cancelled: { label: 'Cancelled', color: 'var(--warning)' },
} satisfies ChartConfig;

const localDay = (iso: string): string => {
  const d = new Date(iso);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
};

const fillBuckets = (daily: DailyOutcome[], days: number) => {
  const byDay = new Map(daily.map(d => [localDay(d.day), d]));
  const today = new Date();
  return Array.from({ length: days }, (_, i) => {
    const d = new Date(today);
    d.setDate(today.getDate() - (days - 1 - i));
    const key = localDay(d.toISOString());
    const hit = byDay.get(key);
    return {
      day: key,
      label: `${d.getDate()}`,
      completed: hit?.completed ?? 0,
      failed: hit?.failed ?? 0,
      cancelled: hit?.cancelled ?? 0,
    };
  });
};

const ToggleGroup = <T extends string>({
  options,
  value,
  onChange,
  label,
}: {
  options: T[];
  value: T;
  onChange: (v: T) => void;
  label: (v: T) => string;
}) => (
  <div className='flex overflow-hidden rounded border text-[11px] font-mono'>
    {options.map(opt => (
      <button
        key={opt}
        type='button'
        onClick={() => onChange(opt)}
        className={cn(
          'px-2.5 py-1 capitalize transition-colors',
          opt === value ? 'bg-foreground text-background' : 'text-muted-foreground hover:bg-muted',
        )}
      >
        {label(opt)}
      </button>
    ))}
  </div>
);

const AgentChartInner = ({ agentId }: { agentId: string }) => {
  const [range, setRange] = useState<Range>('14d');
  const [status, setStatus] = useState<StatusFilter>('all');

  const { data: stats, isLoading } = useAgentStats(agentId);

  const buckets = useMemo(
    () => fillBuckets(stats?.daily ?? [], RANGE_DAYS[range]),
    [stats?.daily, range],
  );

  const hasData = buckets.some(b => b.completed + b.failed + b.cancelled > 0);

  const activeConfig = useMemo<ChartConfig>(() => {
    if (status === 'all') return CHART_CONFIG;
    return { [status]: CHART_CONFIG[status] };
  }, [status]);

  if (isLoading) {
    return <Skeleton className='h-[160px] w-full' />;
  }

  return (
    <div className='space-y-3'>
      <div className='flex flex-wrap items-center justify-between gap-2'>
        <ToggleGroup<StatusFilter>
          options={STATUS_FILTERS}
          value={status}
          onChange={setStatus}
          label={v => v}
        />
        <ToggleGroup<Range> options={RANGES} value={range} onChange={setRange} label={v => v} />
      </div>

      {!hasData ? (
        <div className='flex h-[160px] items-center justify-center text-center'>
          <p className='text-sm text-muted-foreground'>
            <Trans>No finished jobs in this window.</Trans>
          </p>
        </div>
      ) : (
        <ChartContainer config={activeConfig} className='h-[160px] w-full'>
          <AreaChart data={buckets} margin={{ top: 4, right: 8, bottom: 0, left: -16 }}>
            <defs>
              <linearGradient id='grad-completed' x1='0' y1='0' x2='0' y2='1'>
                <stop offset='5%' stopColor='var(--success)' stopOpacity={0.25} />
                <stop offset='95%' stopColor='var(--success)' stopOpacity={0} />
              </linearGradient>
              <linearGradient id='grad-failed' x1='0' y1='0' x2='0' y2='1'>
                <stop offset='5%' stopColor='var(--destructive)' stopOpacity={0.25} />
                <stop offset='95%' stopColor='var(--destructive)' stopOpacity={0} />
              </linearGradient>
              <linearGradient id='grad-cancelled' x1='0' y1='0' x2='0' y2='1'>
                <stop offset='5%' stopColor='var(--warning)' stopOpacity={0.25} />
                <stop offset='95%' stopColor='var(--warning)' stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray='3 3' vertical={false} />
            <XAxis
              dataKey='label'
              tickLine={false}
              axisLine={false}
              tick={{ fontSize: 10 }}
              interval='preserveStartEnd'
            />
            <YAxis
              tickLine={false}
              axisLine={false}
              tick={{ fontSize: 10 }}
              allowDecimals={false}
            />
            <ChartTooltip
              content={
                <ChartTooltipContent
                  labelFormatter={(_, payload) => {
                    const day = payload?.[0]?.payload?.day as string | undefined;
                    return day ?? '';
                  }}
                />
              }
            />
            {(status === 'all' || status === 'completed') && (
              <Area
                type='monotone'
                dataKey='completed'
                stroke='var(--success)'
                strokeWidth={2}
                fill='url(#grad-completed)'
                dot={false}
                activeDot={{ r: 4 }}
              />
            )}
            {(status === 'all' || status === 'failed') && (
              <Area
                type='monotone'
                dataKey='failed'
                stroke='var(--destructive)'
                strokeWidth={2}
                fill='url(#grad-failed)'
                dot={false}
                activeDot={{ r: 4 }}
              />
            )}
            {(status === 'all' || status === 'cancelled') && (
              <Area
                type='monotone'
                dataKey='cancelled'
                stroke='var(--warning)'
                strokeWidth={2}
                fill='url(#grad-cancelled)'
                dot={false}
                activeDot={{ r: 4 }}
              />
            )}
            {status === 'all' && <ChartLegend content={<ChartLegendContent />} />}
          </AreaChart>
        </ChartContainer>
      )}
    </div>
  );
};

export const AgentOutcomesChart = () => {
  const { agents, isLoading: agentsLoading } = useAgents();
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

  const agentId = selectedAgentId ?? agents[0]?.id ?? null;

  if (agentsLoading) {
    return (
      <Card>
        <CardContent className='p-4'>
          <Skeleton className='h-[210px] w-full' />
        </CardContent>
      </Card>
    );
  }

  if (agents.length === 0) {
    return (
      <Card>
        <CardContent className='flex h-[140px] items-center justify-center p-4'>
          <p className='text-sm text-muted-foreground'>
            <Trans>No agents found. Connect an agent to see execution history.</Trans>
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className='pb-2 px-4 pt-4'>
        <div className='flex flex-wrap items-center justify-between gap-3'>
          <CardTitle className='text-base'>
            <Trans>Agent Outcomes</Trans>
          </CardTitle>
          <div className='flex overflow-hidden rounded border text-[11px] font-mono'>
            {agents.map(agent => (
              <button
                key={agent.id}
                type='button'
                onClick={() => setSelectedAgentId(agent.id)}
                className={cn(
                  'flex items-center gap-1.5 px-2.5 py-1 transition-colors',
                  agent.id === agentId
                    ? 'bg-foreground text-background'
                    : 'text-muted-foreground hover:bg-muted',
                )}
              >
                <span
                  className={cn(
                    'h-1.5 w-1.5 rounded-full',
                    agent.connected ? 'bg-success' : 'bg-muted-foreground/50',
                  )}
                />
                {agent.name}
              </button>
            ))}
          </div>
        </div>
      </CardHeader>
      <CardContent className='px-4 pb-4'>
        {agentId && <AgentChartInner agentId={agentId} />}
      </CardContent>
    </Card>
  );
};
