import { msg } from '@lingui/core/macro';

/**
 * Same id the React `PermissionButton` carries, so its French translation
 * follows the port. `lingui extract` does not read `.svelte` — §4.5.
 */
export const gatedButtonMessages = {
  notPermitted: msg`You don't have permission to do this.`,
};
