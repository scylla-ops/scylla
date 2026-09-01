import { Badge, Button } from '@/modules/shared/presentation/ui/shadcn';
import type { JobNodeExecution } from '@/modules/features/jobs/domain/structs/job.struct.ts';
import { ChevronDown, ChevronRight, TerminalSquare } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { useLingui } from '@lingui/react';
import { useState } from 'react';
import { JobLogDisplay } from '@/modules/features/jobs/presentation/ui/jobs-table/jobs-log/JobLogDisplay.tsx';
import { IconButton } from '@shared/presentation/ui';
import { Trans } from '@lingui/react/macro';

type JobNodesListProps = {
  jobId: string;
  nodeExecutions: JobNodeExecution[];
  isExpanded: boolean;
  onCollapse: () => void;
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
 * Display a detailed list of node executions when a job is expanded.
 * Each node can be expanded to show its logs inline (like GitHub Actions).
 */
export const JobNodesList = ({
  jobId,
  nodeExecutions,
  isExpanded,
  onCollapse,
}: JobNodesListProps) => {
  const { _ } = useLingui();
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());

  const toggleNode = (nodeId: string) => {
    setExpandedNodes(prev => {
      const next = new Set(prev);
      if (next.has(nodeId)) next.delete(nodeId);
      else next.add(nodeId);
      return next;
    });
  };

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
              <IconButton icon={ChevronDown} tooltip={<Trans>Collapse</Trans>} onClick={onCollapse} />
              Node Executions ({nodeExecutions.length})
            </h4>
            <div className='space-y-1'>
              {nodeExecutions.map((node, index) => {
                const config = getStatusConfig(node.state);
                const Icon = config.icon;
                const nodeId = node.id || String(index);
                const isNodeExpanded = expandedNodes.has(nodeId);

                return (
                  <div key={index} className='rounded-lg overflow-hidden border'>
                    <Button
                      variant='outline'
                      type='button'
                      onClick={e => {
                        e.stopPropagation();
                        toggleNode(nodeId);
                      }}
                      // hover:scale-100 neutralizes the Button base's
                      // hover:scale-108 — a zoom reads fine on small buttons
                      // but is jarring on a full-width row.
                      className='w-full flex items-center justify-between p-3 h-auto hover:bg-secondary hover:scale-100 transition-colors cursor-pointer rounded-none'
                    >
                      <div className='flex items-center gap-3'>
                        {isNodeExpanded ? (
                          <ChevronDown className='w-4 h-4 text-slate-500' />
                        ) : (
                          <ChevronRight className='w-4 h-4 text-slate-500' />
                        )}
                        <Icon className={`w-5 h-5 ${config.iconClassName}`} />
                        <div className='text-left'>
                          <p className='font-medium text-sm'>{node.id}</p>
                        </div>
                        <Badge variant={config.variant} className='text-xs'>
                          {_(config.label)}
                        </Badge>
                      </div>
                      <div className='flex items-center gap-3 text-right text-sm text-slate-600'>
                        <p className='font-medium'>
                          {calculateDuration(node.startedAt, node.finishedAt)}
                        </p>
                        <TerminalSquare className='w-4 h-4 text-slate-400' />
                      </div>
                    </Button>

                    {isNodeExpanded && (
                      <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: 'auto', opacity: 1 }}
                        transition={{ duration: 0.15 }}
                        className='overflow-hidden'
                      >
                        <div className='p-2'>
                          <JobLogDisplay jobId={jobId} nodeId={nodeId} />
                        </div>
                      </motion.div>
                    )}
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
