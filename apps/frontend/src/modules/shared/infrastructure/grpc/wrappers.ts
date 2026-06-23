//TODO: move this file to the shared module ?

// Helpers for the gRPC boundary. The backend protos wrap every id in a
// strongly typed message (`{ value: string }`) and use
// `google.protobuf.Timestamp` (`{ seconds, nanos }`) for times. The app domain
// keeps plain `string` ids and ISO timestamp strings, so these helpers unwrap
// on read (proto -> domain) and wrap on send (domain -> proto). Keep their use
// confined to infrastructure mappers and data sources.

/** Unwrap a strongly typed id message to its plain string (read side). */
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
export function timestampToIso(timestamp?: {
  seconds: bigint | string | number;
  nanos: number;
}): string {
  if (!timestamp) return '';

  // Sécurité : On s'assure de convertir la valeur en BigInt proprement pour éviter le crash
  // de mix de types (BigInt vs Number)
  const secondsMs = BigInt(timestamp.seconds) * 1000n;
  const nanosMs = BigInt(Math.floor(timestamp.nanos / 1_000_000));
  const totalMillis = Number(secondsMs + nanosMs);

  const date = new Date(totalMillis);

  if (isNaN(date.getTime())) {
    return '';
  }

  // .toISOString() produit STRICTEMENT de l'UTC (ex: "2026-06-22T09:00:00.146Z")
  return date.toISOString();
}

/**
 * Like {@link timestampToIso} but yields `undefined` when the timestamp is
 * absent, for domain fields that are optional (e.g. a node not yet started).
 */
export function timestampToIsoOpt(timestamp?: {
  seconds: bigint | string | number;
  nanos: number;
}): string | undefined {
  if (!timestamp) return undefined;

  if (BigInt(timestamp.seconds) === 0n && timestamp.nanos === 0) {
    return undefined;
  }

  const iso = timestampToIso(timestamp);
  return iso === '' ? undefined : iso;
}
