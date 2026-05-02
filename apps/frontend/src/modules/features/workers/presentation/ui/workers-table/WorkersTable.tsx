import { DataTable } from '@shared/presentation/ui';
import { createWorkerColumns } from '@/modules/features/workers/presentation/ui/workers-table/WorkerColumns.tsx';
import type { Worker } from '@/modules/features/workers/domain/models/worker.model.ts';

interface WorkersTableProps {
  data?: Worker[];
  onView: (workerId: string) => void;
}

export const WorkersTable = ({ data, onView }: WorkersTableProps) => {
  return (
    <DataTable
      columns={createWorkerColumns({ onView })}
      data={data ?? []}
      getRowId={(row, index) => row.agentId || index.toString()}
    />
  );
};
