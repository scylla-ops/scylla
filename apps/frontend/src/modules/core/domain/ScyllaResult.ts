export type ScyllaResult<T> = T | ScyllaError;

export type ScyllaError = {
  message: string;
};
