import { msg } from '@lingui/core/macro';
import { plural } from '@lingui/core/macro';

export const featureHeaderMessages = {
  inTotal: msg`in total`,
  selectAll: msg`Select all`,
  clear: msg`Clear`,
  delete: msg`Delete`,
  notPermitted: msg`You don't have permission to do this.`,
  newEntity: (label: string) => msg`New ${label}`,
  /** `selectedCount`, not `count`: the placeholder name is part of the msgid (see the `.fr` test). */
  itemsDeleted: (selectedCount: number) =>
    msg`${plural(selectedCount, { one: '# item deleted', other: '# items deleted' })}`,
};
