import { t } from '@lingui/core/macro';

/** Class to represent errors in the application.
 * @extends Error
 * **/
export class ScyllaError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);

    Object.setPrototypeOf(this, ScyllaError.prototype);
    Error.captureStackTrace(this, this.constructor);
  }

  private hasCode(cause: unknown): cause is { code: string } {
    return !!cause && typeof cause === 'object' && 'code' in cause;
  }

  public getCode(): string {
    return this.hasCode(this.cause) ? this.cause.code : 'UNKNOWN_ERROR';
  }

  public isNetworkError(): boolean {
    const code = this.getCode();
    if (code === 'UNAVAILABLE') return true;
    return this.cause instanceof Error && this.cause.message.includes('fetch');
  }

  /** The requested resource doesn't exist (gRPC NOT_FOUND). */
  public isNotFound(): boolean {
    return this.getCode() === 'NOT_FOUND';
  }

  /** The caller isn't allowed to see this resource (gRPC PERMISSION_DENIED). */
  public isForbidden(): boolean {
    return this.getCode() === 'PERMISSION_DENIED';
  }

  /** A unique constraint was hit, e.g. a name already taken (gRPC ALREADY_EXISTS). */
  public isAlreadyExists(): boolean {
    return this.getCode() === 'ALREADY_EXISTS';
  }

  /** Message carried by the underlying cause (the gRPC status message), if any. */
  private causeMessage(): string | undefined {
    return this.cause instanceof Error && this.cause.message ? this.cause.message : undefined;
  }

  /**
   * The single message meant for end users. The backend message is surfaced
   * only for codes where it is actionable by the user (validation, conflicts,
   * quotas...); other codes fall back to this error's own wrapper message so
   * internal details never reach a toast.
   */
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

/** Class to represent the result of an operation.
 * @template T - The type of the result.
 * **/
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
