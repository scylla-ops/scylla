import { getQueryClient } from '@platform/query';

/**
 * Running a `*.queries.ts` factory's options without a component.
 *
 * This is the reason the query hooks became options objects: what used to need
 * `renderHook`, a provider stack and a `waitFor` is now a function call.
 *
 * The casts are the price, and they belong here rather than in every test —
 * TanStack types `queryFn` as `QueryFunction | typeof skipToken` and hands it a
 * context object nothing here reads. The conditional types below pull the data
 * type back out of the options so assertions stay typed.
 */

/** `{ queryFn?: QueryFunction<T> | skipToken }` -> `T`. */
type QueryData<TOptions> = TOptions extends { queryFn?: infer TQueryFn }
  ? Awaited<ReturnType<Extract<TQueryFn, (...args: never[]) => unknown>>>
  : unknown;

type MutationData<TOptions> = TOptions extends { mutationFn?: infer TMutationFn }
  ? Awaited<ReturnType<Extract<TMutationFn, (...args: never[]) => unknown>>>
  : unknown;

type MutationVariables<TOptions> = TOptions extends { mutationFn?: infer TMutationFn }
  ? Parameters<Extract<TMutationFn, (...args: never[]) => unknown>>[0]
  : never;

export const runQueryFn = <TOptions extends { queryKey: readonly unknown[] }>(
  options: TOptions,
): Promise<QueryData<TOptions>> => {
  const queryFn = (options as { queryFn?: unknown }).queryFn;
  if (typeof queryFn !== 'function') {
    throw new Error('These options carry no queryFn to run.');
  }

  return (queryFn as (context: unknown) => Promise<QueryData<TOptions>>)({
    queryKey: options.queryKey,
    signal: new AbortController().signal,
    client: getQueryClient(),
    meta: undefined,
  });
};

export const runMutationFn = <TOptions extends { mutationFn?: unknown }>(
  options: TOptions,
  variables: MutationVariables<TOptions>,
): Promise<MutationData<TOptions>> => {
  if (typeof options.mutationFn !== 'function') {
    throw new Error('These options carry no mutationFn to run.');
  }

  return (options.mutationFn as (vars: unknown) => Promise<MutationData<TOptions>>)(variables);
};

/** Fires the success callback, which is where invalidation and toasts live. */
export const runOnSuccess = <TOptions extends { onSuccess?: unknown }>(
  options: TOptions,
  data: MutationData<TOptions>,
  variables: MutationVariables<TOptions>,
): void => {
  const onSuccess = options.onSuccess as
    | ((data: unknown, variables: unknown, context: undefined) => unknown)
    | undefined;

  void onSuccess?.(data, variables, undefined);
};

/**
 * A stand-in for a `*.queries.ts` factory, for a React test that mocks a
 * migrated feature's barrel.
 *
 * Those tests used to stub a hook and hand back `{ data, isLoading }` directly.
 * The hooks are gone: a consumer now passes an options object to `useQuery`, so
 * the stub has to be an options object too. `initialData` is what keeps the
 * test synchronous — without it the first render is always `isLoading`, and
 * every assertion would have to become a `findBy`.
 *
 * ```ts
 * vi.mock('@/modules/features/organization', () => ({
 *   organizationQueries: { mine: () => stubQuery(['organizations'], state.organizations) },
 * }));
 * ```
 */
export const stubQuery = <TData>(
  queryKey: readonly unknown[],
  data: TData | undefined,
  { loading = false }: { loading?: boolean } = {},
) =>
  loading
    ? // Never resolves: "still loading" is a state, not a moment.
      { queryKey, queryFn: () => new Promise<TData>(() => {}) }
    : { queryKey, queryFn: () => Promise.resolve(data as TData), initialData: data as TData };
