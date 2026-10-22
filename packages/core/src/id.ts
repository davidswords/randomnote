export type Clock = () => number;
export type RandomBytes = (length: number) => Uint8Array;

const secureRandomBytes: RandomBytes = (length) =>
  crypto.getRandomValues(new Uint8Array(length));

/**
 * Creates a UUID v7 (RFC 9562): a 48-bit millisecond timestamp followed by
 * random bits, so IDs are unique without coordination and sort by creation
 * time.
 */
export function newId(
  now: Clock = Date.now,
  randomBytes: RandomBytes = secureRandomBytes,
): string {
  const bytes = new Uint8Array(16);
  writeTimestamp(bytes, now());
  bytes.set(randomBytes(10), 6);
  bytes[6] = 0x70 | (bytes[6] & 0x0f);
  bytes[8] = 0x80 | (bytes[8] & 0x3f);
  return formatUuid(bytes);
}

function writeTimestamp(bytes: Uint8Array, milliseconds: number): void {
  let remaining = milliseconds;
  for (let index = 5; index >= 0; index--) {
    bytes[index] = remaining % 256;
    remaining = Math.floor(remaining / 256);
  }
}

function formatUuid(bytes: Uint8Array): string {
  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0"));
  return [
    hex.slice(0, 4),
    hex.slice(4, 6),
    hex.slice(6, 8),
    hex.slice(8, 10),
    hex.slice(10, 16),
  ]
    .map((group) => group.join(""))
    .join("-");
}
