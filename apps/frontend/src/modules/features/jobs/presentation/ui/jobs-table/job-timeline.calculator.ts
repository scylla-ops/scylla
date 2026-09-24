import type { JobNodeExecution } from '../../../domain/structs/job.struct.ts';

export const COLLAPSE_THRESHOLD = 10;

export interface StatusGroup {
  status: string;
  count: number;
  nodes: JobNodeExecution[];
  percent: number;
}

export const shouldCollapse = (nodeCount: number): boolean => nodeCount > COLLAPSE_THRESHOLD;

/** Groups the nodes by status, in first-seen order, with each group's share. */
export const groupByStatus = (nodeExecutions: readonly JobNodeExecution[]): StatusGroup[] => {
  const byStatus = new Map<string, JobNodeExecution[]>();

  for (const node of nodeExecutions) {
    const nodes = byStatus.get(node.state);
    if (nodes) nodes.push(node);
    else byStatus.set(node.state, [node]);
  }

  const total = nodeExecutions.length;

  return [...byStatus.entries()].map(([status, nodes]) => ({
    status,
    count: nodes.length,
    nodes,
    percent: (nodes.length / total) * 100,
  }));
};

/** A node that never started has no id: its position stands in (the URL uses the same). */
export const nodeIdOf = (node: JobNodeExecution, index: number): string => node.id || String(index);
