import { describe, it, expect, vi } from 'vitest';
import { ScyllaError, ScyllaResult } from './scylla-result';

describe('ScyllaError', () => {
  it('defaults to UNKNOWN_ERROR when the cause carries no code', () => {
    const err = new ScyllaError('boom');
    expect(err.getCode()).toBe('UNKNOWN_ERROR');
  });

  it('reads the code off a gRPC-shaped cause', () => {
    const err = new ScyllaError('boom', { cause: { code: 'NOT_FOUND' } });
    expect(err.getCode()).toBe('NOT_FOUND');
    expect(err.isNotFound()).toBe(true);
    expect(err.isForbidden()).toBe(false);
  });

  it('recognizes PERMISSION_DENIED as forbidden', () => {
    const err = new ScyllaError('boom', { cause: { code: 'PERMISSION_DENIED' } });
    expect(err.isForbidden()).toBe(true);
  });

  it('recognizes ALREADY_EXISTS', () => {
    const err = new ScyllaError('boom', { cause: { code: 'ALREADY_EXISTS' } });
    expect(err.isAlreadyExists()).toBe(true);
  });

  it('treats an UNAVAILABLE code as a network error', () => {
    const err = new ScyllaError('boom', { cause: { code: 'UNAVAILABLE' } });
    expect(err.isNetworkError()).toBe(true);
  });

  it('treats a plain fetch failure (no code) as a network error too', () => {
    const err = new ScyllaError('boom', { cause: new Error('failed to fetch') });
    expect(err.isNetworkError()).toBe(true);
  });

  it('does not misclassify an unrelated cause as a network error', () => {
    const err = new ScyllaError('boom', { cause: new Error('validation failed') });
    expect(err.isNetworkError()).toBe(false);
  });

  describe('userMessage', () => {
    it('returns a generic message for network errors, never leaking transport details', () => {
      const err = new ScyllaError('boom', { cause: { code: 'UNAVAILABLE' } });
      expect(err.userMessage()).toBe('Server unreachable');
    });

    it('surfaces the backend message for actionable codes like INVALID_ARGUMENT', () => {
      const err = new ScyllaError('wrapper message', {
        cause: Object.assign(new Error('name must be unique'), { code: 'INVALID_ARGUMENT' }),
      });
      expect(err.userMessage()).toBe('name must be unique');
    });

    it('returns a fixed message for PERMISSION_DENIED regardless of the cause text', () => {
      const err = new ScyllaError('boom', {
        cause: Object.assign(new Error('internal detail'), { code: 'PERMISSION_DENIED' }),
      });
      expect(err.userMessage()).toBe("You don't have permission to perform this action");
    });

    it('falls back to a generic message for an unmapped code with no cause message', () => {
      const err = new ScyllaError('', { cause: { code: 'INTERNAL' } });
      expect(err.userMessage()).toBe('An unexpected error occurred');
    });

    it('falls back to this error’s own message for an unmapped code when there is one', () => {
      const err = new ScyllaError('fallback wrapper message', { cause: { code: 'INTERNAL' } });
      expect(err.userMessage()).toBe('fallback wrapper message');
    });
  });

  it('log() warns with the message and, if present, the cause — never throws', () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const err = new ScyllaError('boom', { cause: new Error('root cause') });
    expect(() => err.log()).not.toThrow();
    expect(warnSpy).toHaveBeenCalledTimes(2);
    warnSpy.mockRestore();
  });
});

describe('ScyllaResult', () => {
  it('success() folds onSuccess with the value', () => {
    const result = ScyllaResult.success(42);
    const out = result.fold({ onSuccess: v => v * 2, onError: () => -1 });
    expect(out).toBe(84);
  });

  it('error() folds onError with the ScyllaError', () => {
    const err = new ScyllaError('boom');
    const result = ScyllaResult.error<number>(err);
    const out = result.fold<string>({ onSuccess: () => 'unreachable', onError: e => e.message });
    expect(out).toBe('boom');
  });

  describe('map', () => {
    it('transforms a success value', () => {
      const out = ScyllaResult.success(2)
        .map(v => v * 10)
        .fold({ onSuccess: v => v, onError: () => -1 });
      expect(out).toBe(20);
    });

    it('short-circuits on an existing error without calling the mapper', () => {
      const fn = vi.fn();
      const err = new ScyllaError('boom');
      ScyllaResult.error<number>(err).map(fn);
      expect(fn).not.toHaveBeenCalled();
    });

    it('catches a throwing mapper and turns it into a ScyllaError', () => {
      const out = ScyllaResult.success(1)
        .map(() => {
          throw new Error('mapper exploded');
        })
        .fold({ onSuccess: () => 'no-error', onError: e => e.message });
      expect(out).toBe('Error mapping value');
    });
  });

  describe('flatMap', () => {
    it('chains two successful results', () => {
      const out = ScyllaResult.success(2)
        .flatMap(v => ScyllaResult.success(v + 1))
        .fold({ onSuccess: v => v, onError: () => -1 });
      expect(out).toBe(3);
    });

    it('propagates an existing error without invoking the next step', () => {
      const fn = vi.fn();
      const err = new ScyllaError('boom');
      ScyllaResult.error<number>(err).flatMap(fn);
      expect(fn).not.toHaveBeenCalled();
    });

    it('catches a throwing step and turns it into a ScyllaError', () => {
      const out = ScyllaResult.success(1)
        .flatMap<number>(() => {
          throw new Error('flatMap exploded');
        })
        .fold({ onSuccess: () => 'no-error', onError: e => e.message });
      expect(out).toBe('Error during flatMap operation');
    });
  });

  describe('flatMapAsync', () => {
    it('chains an async success', async () => {
      const out = await ScyllaResult.success(2)
        .flatMapAsync(v => Promise.resolve(ScyllaResult.success(v + 1)))
        .then(r => r.fold({ onSuccess: v => v, onError: () => -1 }));
      expect(out).toBe(3);
    });

    it('turns a rejected async step into a ScyllaError instead of rejecting', async () => {
      const result = await ScyllaResult.success(1).flatMapAsync<number>(() =>
        Promise.reject(new Error('async exploded')),
      );
      const out = result.fold({ onSuccess: () => 'no-error', onError: e => e.message });
      expect(out).toBe('Error during flatMapAsync operation');
    });
  });

  describe('unwrap', () => {
    it('returns the value on success', () => {
      expect(ScyllaResult.success('ok').unwrap()).toBe('ok');
    });

    it('throws the ScyllaError on error', () => {
      const err = new ScyllaError('boom');
      expect(() => ScyllaResult.error(err).unwrap()).toThrow(err);
    });
  });

  describe('try / tryAsync', () => {
    it('try() wraps a successful synchronous call', () => {
      const out = ScyllaResult.try(() => 5, 'should not appear').unwrap();
      expect(out).toBe(5);
    });

    it('try() wraps a throwing call into a ScyllaError with the given message', () => {
      const result = ScyllaResult.try(() => {
        throw new Error('nope');
      }, 'wrapped message');
      const out = result.fold({ onSuccess: () => 'no-error', onError: e => e.message });
      expect(out).toBe('wrapped message');
    });

    it('tryAsync() wraps a resolved promise', async () => {
      const result = await ScyllaResult.tryAsync(() => Promise.resolve('ok'), 'should not appear');
      expect(result.unwrap()).toBe('ok');
    });

    it('tryAsync() wraps a rejected promise into a ScyllaError with the given message', async () => {
      const result = await ScyllaResult.tryAsync(
        () => Promise.reject(new Error('nope')),
        'wrapped async message',
      );
      const out = result.fold({ onSuccess: () => 'no-error', onError: e => e.message });
      expect(out).toBe('wrapped async message');
    });
  });
});
