import { Badge } from '@/modules/shared/presentation/ui/shadcn';
import type { JobNodeResponse } from '@/generated/job.ts';
import { ChevronDown } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { getStatusConfig } from '@shared/utils/status-config.ts';

type JobNodesListProps = {
  nodeExecutions: JobNodeResponse[];
  isExpanded: boolean;
};

const calculateDuration = (startedAt?: string, finishedAt?: string): string => {
  if (!startedAt) return '-';
  const start = new Date(startedAt).getTime();
  const end = finishedAt ? new Date(finishedAt).getTime() : Date.now();
  const durationMs = end - start;
  const seconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) return `${hours}h ${minutes % 60}m`;
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
  return `${seconds}s`;
};

/**
 * Display a detailed list of node executions when a job is expanded
 */
export const JobNodesList = ({ nodeExecutions, isExpanded }: JobNodesListProps) => {
  return (
    <AnimatePresence>
      {isExpanded && (
        <motion.div
          initial={{ height: 0, opacity: 0 }}
          animate={{ height: 'auto', opacity: 1 }}
          exit={{ height: 0, opacity: 0 }}
          transition={{ duration: 0.2 }}
          className='overflow-hidden'
        >
          <div className='mt-4 px-4 pb-4 border-t pt-4'>
            <h4 className='text-sm font-semibold mb-3 flex items-center gap-2'>
              <ChevronDown className='w-4 h-4' />
              Node Executions ({nodeExecutions.length})
            </h4>
            <div className='space-y-2'>
              {nodeExecutions.map((node, index) => {
                const config = getStatusConfig(node.state);
                const Icon = config.icon;

                return (
                  <div
                    key={index}
                    className='flex items-center justify-between p-3 bg-slate-50 rounded-lg'
                  >
                    <div className='flex items-center gap-3'>
                      <Icon className={`w-5 h-5 ${config.iconClassName}`} />
                      <div>
                        <p className='font-medium text-sm'>{node.nodeId}</p>
                        <Badge variant={config.variant} className='text-xs mt-1'>
                          {config.label}
                        </Badge>
                      </div>
                    </div>
                    <div className='text-right text-sm text-slate-600'>
                      <p className='font-medium'>
                        {calculateDuration(node.startedAt, node.finishedAt)}
                      </p>
                      {node.startedAt && (
                        <p className='text-xs text-slate-500'>
                          Started: {new Date(node.startedAt).toLocaleTimeString()}
                        </p>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
