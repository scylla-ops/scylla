import { useEffect, useMemo, useState } from 'react';
import { useWorkers } from '@/modules/features/workers/presentation/hooks/use-workers.ts';
import { FeatureHeader } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { Input } from '@shadcn';
import { formatDate } from '@shared/utils/date-utils.ts';
import type { Worker } from '@/modules/features/workers/domain/models/worker.model.ts';
import { Trans } from '@lingui/react/macro';

const statusOptions = ['All', 'Connected', 'Disconnected', 'Error'] as const;

type StatusFilter = (typeof statusOptions)[number];

const getStatusDotClass = (status: string) => {
  const normalized = status.toLowerCase();

  if (normalized === 'connected') return 'bg-emerald-500';
  if (normalized === 'disconnected') return 'bg-slate-400';
  if (normalized === 'error') return 'bg-red-500';
  return 'bg-amber-500';
};

const formatWorkerId = (id: string) => (id.length > 28 ? `${id.substring(0, 28)}...` : id);

export const WorkersPage = () => {
  const { workers, isLoading, isError, error, searchTerm, setSearchTerm } = useWorkers();
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('All');
  const [selectedWorkerId, setSelectedWorkerId] = useState<string | null>(null);

  useEffect(() => {
    if (!selectedWorkerId && workers.length > 0) {
      setSelectedWorkerId(workers[0].agentId);
    }
  }, [workers, selectedWorkerId]);

  const filteredWorkers = useMemo(() => {
    return workers.filter(worker => {
      const matchesSearch =
        worker.hostname.toLowerCase().includes(searchTerm.toLowerCase()) ||
        worker.agentId.toLowerCase().includes(searchTerm.toLowerCase()) ||
        worker.status.toLowerCase().includes(searchTerm.toLowerCase());

      const matchesStatus =
        statusFilter === 'All' || worker.status.toLowerCase() === statusFilter.toLowerCase();

      return matchesSearch && matchesStatus;
    });
  }, [workers, searchTerm, statusFilter]);

  useEffect(() => {
    if (!filteredWorkers.length) {
      setSelectedWorkerId(null);
      return;
    }

    if (!selectedWorkerId || !filteredWorkers.some(worker => worker.agentId === selectedWorkerId)) {
      setSelectedWorkerId(filteredWorkers[0].agentId);
    }
  }, [filteredWorkers, selectedWorkerId]);

  const selectedWorker = selectedWorkerId
    ? workers.find(worker => worker.agentId === selectedWorkerId)
    : undefined;

  if (isLoading) return <></>;
  if (isError) {
    console.log(error);
    return <ErrorState message='Error loading workers' />;
  }

  return (
    <div className='space-y-4 w-full p-2'>
      <FeatureHeader count={workers?.length ?? 0} label='Worker' onNew={undefined} />

      <div className='grid gap-4 lg:grid-cols-[400px_1fr]'>
        <div className='space-y-4'>
          <div className='rounded-xl border border-slate-200 bg-white p-5 shadow-sm'>
            <div className='flex flex-col gap-4'>
              <Input
                placeholder='Search workers by hostname, ID, or status...'
                value={searchTerm}
                onChange={e => setSearchTerm(e.target.value)}
                className='w-full'
              />
              <div className='flex flex-wrap gap-2'>
                {statusOptions.map(option => (
                  <button
                    key={option}
                    type='button'
                    onClick={() => setStatusFilter(option)}
                    className={`rounded-full border px-3 py-2 text-sm transition-all ${
                      statusFilter === option
                        ? 'border-slate-800 bg-slate-900 text-white'
                        : 'border-slate-200 bg-white text-slate-600 hover:border-slate-300 hover:bg-slate-50'
                    }`}
                  >
                    {option}
                  </button>
                ))}
              </div>
            </div>
          </div>

          <div className='rounded-xl border border-slate-200 bg-white p-4 shadow-sm'>
            <div className='mb-4 flex items-center justify-between gap-2 text-sm text-slate-500'>
              <span>
                <Trans>{filteredWorkers.length} workers found</Trans>
              </span>
              <span className='hidden sm:inline'>
                <Trans>Click a worker to view details.</Trans>
              </span>
            </div>

            <div className='grid gap-3'>
              {filteredWorkers.map(worker => {
                const isSelected = worker.agentId === selectedWorkerId;
                return (
                  <button
                    key={worker.agentId}
                    type='button'
                    onClick={() => setSelectedWorkerId(worker.agentId)}
                    className={`group w-full rounded-3xl border p-4 text-left transition ${
                      isSelected
                        ? 'border-slate-800 bg-slate-100 shadow-sm'
                        : 'border-slate-200 bg-white hover:border-slate-300 hover:shadow-sm'
                    }`}
                  >
                    <div className='flex items-start justify-between gap-4'>
                      <div>
                        <p className='text-base font-semibold text-slate-900'>{worker.hostname}</p>
                      </div>
                      <span
                        className={`mt-1 inline-flex h-3.5 w-3.5 rounded-full ${getStatusDotClass(worker.status)}`}
                        aria-label={worker.status}
                      />
                    </div>

                    <div className='mt-4 grid gap-2 text-sm text-slate-600'>
                      <div>
                        <span className='font-medium text-slate-900'>ID: </span>
                        <span className='font-mono'>{formatWorkerId(worker.agentId)}</span>
                      </div>
                      <div>
                        <span className='font-medium text-slate-900'>
                          <Trans>Last seen:</Trans>
                        </span>{' '}
                        {formatDate(worker.lastSeenAt)}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        </div>

        <div className='space-y-4'>
          <div className='rounded-xl border border-slate-200 bg-white p-6 shadow-sm h-full'>
            <div className='flex items-center justify-between gap-4'>
              <div>
                <h2 className='text-xl font-semibold tracking-tight'>
                  <Trans>Worker details</Trans>
                </h2>
                <p className='text-sm text-muted-foreground'>
                  <Trans>Select a worker from the list to view its full information.</Trans>
                </p>
              </div>
            </div>

            {!selectedWorker ? (
              <div className='mt-8 rounded-2xl border border-dashed border-slate-200 p-8 text-center text-sm text-slate-500'>
                <Trans>No worker selected.</Trans>
              </div>
            ) : (
              <div className='mt-6 space-y-5'>
                <div className='rounded-3xl border border-slate-200 bg-slate-50 p-5'>
                  <div className='flex items-center justify-between gap-4'>
                    <div>
                      <p className='text-sm uppercase tracking-[0.16em] text-slate-500'>
                        <Trans>Hostname</Trans>
                      </p>
                      <p className='text-2xl font-semibold text-slate-900'>
                        {selectedWorker.hostname}
                      </p>
                    </div>
                    <div className='flex items-center gap-2'>
                      <span
                        className={`inline-flex h-3.5 w-3.5 rounded-full ${getStatusDotClass(selectedWorker.status)}`}
                      />
                      <span className='text-sm text-slate-600'>{selectedWorker.status}</span>
                    </div>
                  </div>
                </div>

                <div className='grid gap-4 sm:grid-cols-2'>
                  <div className='rounded-3xl border border-slate-200 bg-white p-5'>
                    <p className='text-sm text-slate-500'>
                      <Trans>Agent ID</Trans>
                    </p>
                    <p className='mt-2 font-mono text-sm text-slate-900'>
                      {selectedWorker.agentId}
                    </p>
                  </div>
                  <div className='rounded-3xl border border-slate-200 bg-white p-5'>
                    <p className='text-sm text-slate-500'>
                      <Trans>Last seen</Trans>
                    </p>
                    <p className='mt-2 text-sm text-slate-900'>
                      {formatDate(selectedWorker.lastSeenAt)}
                    </p>
                  </div>
                </div>

                <div className='rounded-3xl border border-slate-200 bg-white p-5 space-y-4'>
                  <div>
                    <p className='text-sm text-slate-500'>
                      <Trans>Created at</Trans>
                    </p>
                    <p className='mt-2 text-sm text-slate-900'>
                      {formatDate(selectedWorker.createdAt)}
                    </p>
                  </div>
                  <div>
                    <p className='text-sm text-slate-500'>
                      <Trans>Updated at</Trans>
                    </p>
                    <p className='mt-2 text-sm text-slate-900'>
                      {formatDate(selectedWorker.updatedAt)}
                    </p>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
