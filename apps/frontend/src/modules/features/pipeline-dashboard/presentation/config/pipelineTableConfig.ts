/**
 * Defines the configuration for the pipeline table columns, including their IDs, widths, class names, labels, and whether they should have a separator. This configuration is used to render the pipeline table in the dashboard with consistent styling and structure.
 */
export type PipelineColumnConfig = {
  id: 'status' | 'history' | 'metadata' | 'actions';
  width?: string;
  className?: string;
  label: string;
  noSeparator?: boolean;
};

/**
 * Config of the pipeline table columns.
 */
export const PIPELINE_COLUMNS: PipelineColumnConfig[] = [
  {
    id: 'status',
    width: '20%',
    className: 'flex items-center gap-3 shrink-0',
    label: 'Status',
  },
  {
    id: 'history',
    width: '35%',
    className: 'flex items-center justify-center shrink-0',
    label: 'History',
  },
  {
    id: 'metadata',
    width: '15%',
    className: 'flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
    label: 'Last execution',
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
export const getColumnConfig = (id: PipelineColumnConfig['id']): PipelineColumnConfig => {
  const config = PIPELINE_COLUMNS.find(col => col.id === id);
  if (!config) {
    throw new Error(`Column configuration not found for id: ${id}`);
  }
  return config;
};
