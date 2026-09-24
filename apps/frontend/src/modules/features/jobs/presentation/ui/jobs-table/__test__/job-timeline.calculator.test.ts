// @vitest-environment node
import { describe, it, expect } from 'vitest';
import type { JobNodeExecution } from '../../../../domain/structs/job.struct.ts';
import { COLLAPSE_THRESHOLD, groupByStatus, nodeIdOf, shouldCollapse } from '../job-timeline.calculator.ts';

const node = (id: string, state: string): JobNodeExecution => ({ id, state });

describe('shouldCollapse', () => {
  it('draws one segment per node right up to the threshold', () => {
    expect(shouldCollapse(COLLAPSE_THRESHOLD)).toBe(false);
  });

  it('groups by status past it, where per-node segments stop being readable', () => {
    expect(shouldCollapse(COLLAPSE_THRESHOLD + 1)).toBe(true);
  });
});

describe('groupByStatus', () => {
  it('keeps one group per status, in the order the statuses first appear', () => {
    const groups = groupByStatus([
      node('a', 'completed'),
      node('b', 'failed'),
      node('c', 'completed'),
    ]);

    expect(groups.map(group => group.status)).toEqual(['completed', 'failed']);
    expect(groups[0].count).toBe(2);
    expect(groups[0].nodes.map(item => item.id)).toEqual(['a', 'c']);
  });

  it("gives each group its share of the bar, which is the segment's width", () => {
    const groups = groupByStatus([
      node('a', 'completed'),
      node('b', 'completed'),
      node('c', 'completed'),
      node('d', 'failed'),
    ]);

    expect(groups[0].percent).toBe(75);
    expect(groups[1].percent).toBe(25);
  });

  it('adds the shares up to the whole bar, leaving no gap', () => {
    const groups = groupByStatus([
      node('a', 'completed'),
      node('b', 'failed'),
      node('c', 'running'),
    ]);

    const total = groups.reduce((sum, group) => sum + group.percent, 0);
    expect(total).toBeCloseTo(100);
  });

  it('groups nothing into nothing, rather than dividing by zero', () => {
    expect(groupByStatus([])).toEqual([]);
  });
});

describe('nodeIdOf', () => {
  it('addresses a node by its id', () => {
    expect(nodeIdOf(node('build', 'completed'), 3)).toBe('build');
  });

  it('falls back to the position for a node the backend left unnamed', () => {
    // A node that never started has no id: its position stands in, as in the URL.
    expect(nodeIdOf(node('', 'pending'), 2)).toBe('2');
  });
});
