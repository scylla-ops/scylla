import type { JobNodeExecution } from '../../../domain/structs/job.struct.ts';

/** Above this many nodes the bar stops showing one segment per node. */
export const COLLAPSE_THRESHOLD = 10;

export interface StatusGroup {
  status: string;
  count: number;
  nodes: JobNodeExecution[];
  /** Share of the whole timeline, in percent — the segment's width. */
  percent: number;
}

/** Whether a timeline of this size is drawn per node or per status. */
export const shouldCollapse = (nodeCount: number): boolean => nodeCount > COLLAPSE_THRESHOLD;

/**
 * Groups node executions by status, in first-seen order, with each group's
 * share of the whole.
 *
 * A pipeline with two hundred nodes cannot show a readable segment per node, so
 * past {@link COLLAPSE_THRESHOLD} the bar shows one proportional segment per
 * status instead. The grouping is pure arithmetic over the list, which is why
 * it is a `.ts` tested in `node` rather than a `$derived` in the component —
 * the percentages are the part worth pinning.
 */
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

/**
 * The id a node execution is addressed by.
 *
 * The backend leaves `id` empty for a node that never started, and the position
 * is the only thing left to tell two of those apart — the same fallback the URL
 * and the log panels use, so a link built from one matches the other.
 */
export const nodeIdOf = (node: JobNodeExecution, index: number): string => node.id || String(index);
