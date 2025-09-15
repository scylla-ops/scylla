export type ScyllaResult<T> = { ok: true; value: T } | { ok: false; error: ScyllaError };

/** @brief represent a Scylla domain error,
 * For example, if the server answers correctly but the action is not allowed
 * (user already exists etc.)
 * **/
export type ScyllaError = {
  message: string;
};
