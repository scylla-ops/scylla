import { msg } from '@lingui/core/macro';
import { plural } from '@lingui/core/macro';

export const featureHeaderMessages = {
  inTotal: msg`in total`,
  selectAll: msg`Select all`,
  clear: msg`Clear`,
  delete: msg`Delete`,
  notPermitted: msg`You don't have permission to do this.`,
  newEntity: (label: string) => msg`New ${label}`,
  /**
   * A plural arm and a placeholder, so the French form is structurally
   * different from the English — `FeatureHeader.fr.test.ts` renders it.
   *
   * The parameter is named `selectedCount`, not `count`, because **the
   * placeholder name is part of the msgid**: `{count, plural, …}` and
   * `{selectedCount, plural, …}` are two different messages. Renaming it here
   * would have forked the French translation and left this arm rendering in
   * English, with every catalog still looking complete and `i18n:collisions`
   * still at zero. A ported message keeps the original's variable names.
   */
  itemsDeleted: (selectedCount: number) =>
    msg`${plural(selectedCount, { one: '# item deleted', other: '# items deleted' })}`,
};
