/**
 * Configuration for the jobs table columns
 */
export type JobColumnConfig = {
  id: 'status' | 'jobId' | 'timeline' | 'duration' | 'created' | 'actions';
  width?: string;
  className?: string;
  label: string;
  noSeparator?: boolean;
};

/**
 * Config of the jobs table columns.
 */
export const JOB_COLUMNS: JobColumnConfig[] = [
  {
    id: 'status',
    width: '15%',
    className: 'flex items-center gap-3 shrink-0',
    label: 'Status',
  },
  {
    id: 'jobId',
    width: '20%',
    className: 'flex items-center gap-2 shrink-0',
    label: 'Job ID',
  },
  {
    id: 'timeline',
    width: '30%',
    className: 'flex items-center justify-center shrink-0',
    label: 'Timeline',
  },
  {
    id: 'duration',
    width: '15%',
    className: 'flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
    label: 'Duration',
  },
  {
    id: 'created',
    width: '15%',
    className: 'flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
    label: 'Created',
  },
  {
    id: 'actions',
    className: 'flex justify-center items-center gap-1 flex-1 min-w-[48px]',
    label: 'Actions',
    noSeparator: true,
  },
];

/**
 * Get the column configuration by ID.
 */
export const getColumnConfig = (id: JobColumnConfig['id']): JobColumnConfig => {
  const config = JOB_COLUMNS.find(col => col.id === id);
  if (!config) {
    throw new Error(`Column configuration not found for id: ${id}`);
  }
  return config;
};

