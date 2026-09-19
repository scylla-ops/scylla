import { msg } from '@lingui/core/macro';

/**
 * `lingui extract` does not read `.svelte` files, so every message the sign-in
 * screen shows is declared here and referenced through `t()` — see
 * `refacto_svelte.md` §4.5.
 *
 * The ids are the English source strings and are unchanged from the React
 * components these replaced, which is what carries the French translations over
 * untouched.
 */
export const loginMessages = {
  title: msg`Login to your account`,
  description: msg`Enter your username below to login to your account`,
  username: msg`Username`,
  usernamePlaceholder: msg`username`,
  password: msg`Password`,
  passwordPlaceholder: msg`••••••••`,
  submit: msg`Login`,
};
