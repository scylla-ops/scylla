// Helpers for the gRPC boundary. The backend protos wrap every id in a
// strongly-typed message (`{ value: string }`) and use
// `google.protobuf.Timestamp` (`{ seconds, nanos }`) for times. The app domain
// keeps plain `string` ids and ISO timestamp strings, so these helpers unwrap
// on read (proto -> domain) and wrap on send (domain -> proto). Keep their use
// confined to infrastructure mappers and data sources.

/** Unwrap a strongly-typed id message to its plain string (read side). */
export function idValue(wrapper?: { value: string }): string {
  return wrapper?.value ?? '';
}

/** Wrap a plain string id into a `{ value }` message (write side). */
export function wrapId(value: string): { value: string } {
  return { value };
}

/**
 * Wrap an optional string id. Returns `undefined` when the id is absent so the
 * proto3 message-presence semantics are preserved (an unset optional field).
 */
export function wrapIdOpt(value?: string): { value: string } | undefined {
  return value ? { value } : undefined;
}

/**
 * Format a `google.protobuf.Timestamp` to the ISO-8601 string the app domain
 * expects. Returns an empty string when the timestamp is absent (matching the
 * previous "always a string" domain contract for required fields).
 */
export function timestampToIso(timestamp?: { seconds: bigint | string; nanos: number }): string {
  if (!timestamp) return '';
  const millis = Number(timestamp.seconds) * 1000 + Math.floor(timestamp.nanos / 1_000_000);
  return new Date(millis).toISOString();
}

/**
 * Like {@link timestampToIso} but yields `undefined` when the timestamp is
 * absent, for domain fields that are optional (e.g. a node not yet started).
 */
export function timestampToIsoOpt(
  timestamp?: { seconds: bigint | string; nanos: number },
): string | undefined {
  return timestamp ? timestampToIso(timestamp) : undefined;
}
