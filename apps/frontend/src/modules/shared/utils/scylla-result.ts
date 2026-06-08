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
      return new ScyllaResult<U>(fn(this._value));
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
