// The one part of the Web Crypto API that core uses. It exists in browsers,
// the Tauri web view and Node, so core needs neither DOM nor Node types.
declare const crypto: {
  getRandomValues<T extends Uint8Array>(array: T): T;
};
