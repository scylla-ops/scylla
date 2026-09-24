import { msg } from '@lingui/core/macro';

/**
 * `lingui extract` does not read `.svelte`, so these live here — see
 * `refacto_svelte.md` §4.5. The ids are byte-identical to the ones the React
 * component carried, which is what keeps their French translations attached.
 */
export const secretRevealMessages = {
  copySecret: msg`Copy your secret`,
  shownOnce: msg`shown once`,
  secretCopied: msg`Secret copied`,
  reveal: msg`Reveal`,
  revealFirst: msg`Reveal the secret first`,
  noSecondChance: msg`You won't see this secret again.`,
  done: msg`Done`,
};
