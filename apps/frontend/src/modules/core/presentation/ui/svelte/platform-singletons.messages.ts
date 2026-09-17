import { msg } from '@lingui/core/macro';

/**
 * Messages for the Phase 0 singleton test.
 *
 * Declared here rather than in the `.svelte` file on purpose: this is the
 * convention every migrated component follows, because `lingui extract` does not
 * read `.svelte` and would drop the message without failing anything. Pinning it
 * in the test that proves the bridge works keeps the rule visible.
 */
export const platformSingletonsMessages = {
  greeting: msg`Island is mounted`,
};
