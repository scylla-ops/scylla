import { createMutation } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { navigateTo } from '@platform/context';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { LoginModule } from '../login.module.ts';

export interface Credentials {
  login: string;
  password: string;
}

/**
 * The sign-in screen's view model — what `use-login.ts` used to be.
 *
 * **Construct it during a component's initialisation, never in a handler.**
 * `createMutation` installs an `$effect.pre`, which needs an owner; outside one
 * Svelte throws `effect_orphan`. That is true of every view model here that
 * holds a query or a mutation.
 *
 * The domain is resolved here rather than in the page: a component never
 * reaches a repository, which is the rule `use-<feature>-domain.ts` used to
 * carry and the reason there is no such file any more.
 */
export class LoginState {
  // ---------------------------------------------------------------------------
  // 1. DATA FETCHING
  // ---------------------------------------------------------------------------
  private readonly repository = getModuleDomain<typeof LoginModule.domain>('login')
    .loginRepository;

  private readonly mutation = createMutation<void, ScyllaError, Credentials>(() => ({
    mutationFn: async ({ login, password }: Credentials) =>
      (await this.repository.login(login, password)).unwrap(),
    // Where the user lands is the shell's business, not this module's: `/` is
    // the redirect that resolves to whatever organization they can reach.
    // `replace` keeps the sign-in page out of the back stack.
    onSuccess: () => navigateTo('/', { replace: true }),
    // No `onError`: the global MutationCache handler already toasts, and the
    // data source has re-coded UNAUTHENTICATED so this one does not sign out.
  }));

  // ---------------------------------------------------------------------------
  // 2. OUTPUTS
  // ---------------------------------------------------------------------------
  get isPending(): boolean {
    return this.mutation.isPending;
  }

  /**
   * True from the moment the credentials are accepted until the redirect has
   * torn the page down — the window the loading screen fills.
   */
  get isSuccess(): boolean {
    return this.mutation.isSuccess;
  }

  // ---------------------------------------------------------------------------
  // 3. ACTIONS
  // ---------------------------------------------------------------------------
  submit = (login: string, password: string): void => {
    this.mutation.mutate({ login, password });
  };
}
