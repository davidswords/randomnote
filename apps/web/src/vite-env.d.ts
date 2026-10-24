/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** The randomtask server, such as http://127.0.0.1:8080. Unset, accounts stay in the browser. */
  readonly VITE_RANDOMTASK_SERVER?: string;
}

/** The Rust app compiled to WebAssembly, generated into src/wasm by `npm run build:wasm`. */
declare module "randomtask-web-wasm" {
  export default function init(): Promise<unknown>;
  export class WebApp {
    static open(server?: string): Promise<WebApp>;
    invoke(command: string, args: string): Promise<string>;
  }
}

/** The app's version, from the desktop app's package.json. */
declare const __APP_VERSION__: string;
