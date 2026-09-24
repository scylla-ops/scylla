// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { buildGridTemplate, minTableWidthOf } from '../data-table.ts';

/**
 * The grid maths, tested without a DOM.
 *
 * The React twin asserts these through a rendered table and reads
 * `style.gridTemplateColumns` back off a row. Same rules, but jsdom is not
 * needed to decide what `minmax()` a column gets — `DataTable.test.ts` only has
 * to prove the string reaches the element.
 */
describe('buildGridTemplate', () => {
  it('lets a column with no size absorb the leftover space, floored at its minSize', () => {
    expect(buildGridTemplate([{ minSize: 120 }])).toBe('minmax(120px, 1fr)');
  });

  it('lets a column with no minSize shrink to nothing', () => {
    expect(buildGridTemplate([{}])).toBe('minmax(0px, 1fr)');
  });

  it('keeps sized columns in px while at least one column is flexible', () => {
    expect(buildGridTemplate([{ size: 200, minSize: 80 }, {}])).toBe(
      'minmax(80px, 200px) minmax(0px, 1fr)',
    );
  });

  it('switches every track to fr once no column is flexible, so the table still fills its container', () => {
    expect(buildGridTemplate([{ size: 2, minSize: 80 }, { size: 1 }])).toBe(
      'minmax(80px, 2fr) minmax(0px, 1fr)',
    );
  });
});

describe('minTableWidthOf', () => {
  it('sums the declared minimums', () => {
    expect(minTableWidthOf([{ minSize: 120 }, { minSize: 80 }])).toBe(200);
  });

  it('is zero when no column declares one, which is what leaves min-width unset', () => {
    expect(minTableWidthOf([{}, {}])).toBe(0);
  });
});
