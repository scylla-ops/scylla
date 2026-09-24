import { describe, it, expect, afterEach, beforeEach, vi } from 'vitest';
import { flushSync } from 'svelte';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { createResourceError } from '../resource-error.svelte.ts';

const toastError = vi.fn();
vi.mock('svelte-sonner', () => ({ toast: { error: (message: string) => toastError(message) } }));

let navigator: ReturnType<typeof installTestNavigator>;

beforeEach(() => {
  toastError.mockClear();
  navigator = installTestNavigator();
});

afterEach(() => navigator.restore());

const withCode = (code: string, message: string) =>
  new ScyllaError(message, { cause: { code } });
const notFound = () => withCode('NOT_FOUND', 'gone');

describe('createResourceError', () => {
  it('sits still while the query is fine', () => {
    const cleanup = $effect.root(() => {
      const state = createResourceError({
        error: () => undefined,
        redirectTo: '..',
        notFoundMessage: 'App not found',
      });
      flushSync();

      expect(state.redirecting).toBe(false);
      expect(navigator.navigate).not.toHaveBeenCalled();
    });
    cleanup();
  });

  it('toasts and redirects when the resource is gone', () => {
    const cleanup = $effect.root(() => {
      const state = createResourceError({
        error: () => notFound(),
        redirectTo: '..',
        notFoundMessage: 'App not found',
      });
      flushSync();

      expect(state.redirecting).toBe(true);
      expect(toastError).toHaveBeenCalledWith('App not found');
      // `replace`, so the back button does not land on the page that just 404'd.
      expect(navigator.navigate).toHaveBeenCalledWith('..', { replace: true });
    });
    cleanup();
  });

  it('leaves any other failure to the page, which can still show its error state', () => {
    const cleanup = $effect.root(() => {
      const state = createResourceError({
        error: () => withCode('UNAVAILABLE', 'backend is down'),
        redirectTo: '..',
        notFoundMessage: 'App not found',
      });
      flushSync();

      expect(state.redirecting).toBe(false);
      expect(state.scyllaError?.getCode()).toBe('UNAVAILABLE');
      expect(navigator.navigate).not.toHaveBeenCalled();
    });
    cleanup();
  });

  it('reacts to an error that only arrives on a later render', () => {
    const cleanup = $effect.root(() => {
      // The reason `error` is a getter: a query starts undefined and fails
      // later, and taking the value once would never see the failure.
      let error = $state<unknown>(undefined);
      const state = createResourceError({
        error: () => error,
        redirectTo: '..',
        notFoundMessage: 'App not found',
      });
      flushSync();
      expect(state.redirecting).toBe(false);

      error = notFound();
      flushSync();

      expect(state.redirecting).toBe(true);
      expect(navigator.navigate).toHaveBeenCalledWith('..', { replace: true });
    });
    cleanup();
  });
});
