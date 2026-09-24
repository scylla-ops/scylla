import { getQueryClient } from '@platform/query';

/** Runs a `*.queries.ts` factory's options without a component. The casts live here, not in every test. */

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
 * A stand-in for a query factory, for a test that mocks a feature's barrel.
 * `initialData` keeps the first render synchronous.
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
