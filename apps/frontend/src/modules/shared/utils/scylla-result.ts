import { t } from '@lingui/core/macro';
import type { GrpcStatusCode } from '@protobuf-ts/grpcweb-transport';

/** The gRPC status names, plus our own codes (e.g. `INVALID_CREDENTIALS` from `login`). */
export type ScyllaErrorCode =
  | keyof typeof GrpcStatusCode
  | 'UNKNOWN_ERROR'
  | 'INVALID_CREDENTIALS';

export class ScyllaError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);

    Object.setPrototypeOf(this, ScyllaError.prototype);
    Error.captureStackTrace(this, this.constructor);
  }

  private hasCode(cause: unknown): cause is { code: string } {
    return !!cause && typeof cause === 'object' && 'code' in cause;
  }

  /** A cast: the value comes from the wire. An unknown code matches no branch. */
  public getCode(): ScyllaErrorCode {
    return this.hasCode(this.cause) ? (this.cause.code as ScyllaErrorCode) : 'UNKNOWN_ERROR';
  }

  public isNetworkError(): boolean {
    const code = this.getCode();
    if (code === 'UNAVAILABLE') return true;
    return this.cause instanceof Error && this.cause.message.includes('fetch');
  }

  public isNotFound(): boolean {
    return this.getCode() === 'NOT_FOUND';
  }

  public isForbidden(): boolean {
    return this.getCode() === 'PERMISSION_DENIED';
  }

  public isAlreadyExists(): boolean {
    return this.getCode() === 'ALREADY_EXISTS';
  }

  private causeMessage(): string | undefined {
    return this.cause instanceof Error && this.cause.message ? this.cause.message : undefined;
  }

  /** The backend message only for codes the user can act on; otherwise this error's own message. */
  public userMessage(): string {
    if (this.isNetworkError()) {
      return t`Server unreachable`;
    }
    switch (this.getCode()) {
      case 'INVALID_ARGUMENT':
      case 'FAILED_PRECONDITION':
      case 'ALREADY_EXISTS':
      case 'RESOURCE_EXHAUSTED':
      case 'NOT_FOUND':
        return this.causeMessage() ?? this.message;
      case 'PERMISSION_DENIED':
        return t`You don't have permission to perform this action`;
      default:
        return (this.causeMessage() ?? this.message) || t`An unexpected error occurred`;
    }
  }

  public log(): void {
    console.warn(`>[${this.constructor.name}]:`, this.message);
    if (this.cause) {
      console.warn('Original cause:', this.cause);
    }
  }
}

export class ScyllaResult<T> {
  constructor(private readonly _value: T | ScyllaError) {}

  public fold<U>(callbacks: { onSuccess: (value: T) => U; onError: (error: ScyllaError) => U }): U {
    if (this._value instanceof ScyllaError) {
      return callbacks.onError(this._value);
    } else {
      return callbacks.onSuccess(this._value);
    }
  }

  public map<U>(fn: (value: T) => U): ScyllaResult<U> {
    if (this._value instanceof ScyllaError) {
      return new ScyllaResult<U>(this._value);
    } else {
      try {
        return new ScyllaResult<U>(fn(this._value));
      } catch (error) {
        return new ScyllaResult<U>(new ScyllaError('Error mapping value', { cause: error }));
      }
    }
  }

  public flatMap<U>(fn: (value: T) => ScyllaResult<U>): ScyllaResult<U> {
    if (this._value instanceof ScyllaError) {
      return new ScyllaResult<U>(this._value);
    }
    try {
      return fn(this._value);
    } catch (error) {
      return new ScyllaResult<U>(
        new ScyllaError('Error during flatMap operation', { cause: error }),
      );
    }
  }

  public async flatMapAsync<U>(
    fn: (value: T) => Promise<ScyllaResult<U>>,
  ): Promise<ScyllaResult<U>> {
    if (this._value instanceof ScyllaError) {
      return new ScyllaResult<U>(this._value);
    }
    try {
      return await fn(this._value);
    } catch (error) {
      return new ScyllaResult<U>(
        new ScyllaError('Error during flatMapAsync operation', { cause: error }),
      );
    }
  }

  /** For a call where a generic code means something more precise. A success passes through. */
  public mapError(fn: (error: ScyllaError) => ScyllaError): ScyllaResult<T> {
    return this._value instanceof ScyllaError ? new ScyllaResult<T>(fn(this._value)) : this;
  }

  public unwrap(): T {
    if (this._value instanceof ScyllaError) {
      throw this._value;
    }
    return this._value;
  }

  public static try<T>(fn: () => T, errorMessage: string): ScyllaResult<T> {
    try {
      return new ScyllaResult<T>(fn());
    } catch (error) {
      return new ScyllaResult<T>(new ScyllaError(errorMessage, { cause: error }));
    }
  }

  public static async tryAsync<T>(
    fn: () => Promise<T>,
    errorMessage: string,
  ): Promise<ScyllaResult<T>> {
    try {
      const value = await fn();
      return new ScyllaResult<T>(value);
    } catch (error) {
      return new ScyllaResult<T>(new ScyllaError(errorMessage, { cause: error }));
    }
  }

  public static success<T>(value: T): ScyllaResult<T> {
    return new ScyllaResult<T>(value);
  }

  public static error<T>(error: ScyllaError): ScyllaResult<T> {
    return new ScyllaResult<T>(error);
  }
}
