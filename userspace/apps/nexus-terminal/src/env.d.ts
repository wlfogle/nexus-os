/// <reference types="vite/client" />

declare global {
  interface Window {
    __TAURI__?: {
      core: any;
      event: any;
      path: any;
      fs: any;
    };
  }
}

interface ImportMetaEnv {
  readonly VITE_APP_TITLE: string;
  readonly VITE_APP_VERSION: string;
  readonly VITE_API_BASE_URL: string;
  readonly VITE_OLLAMA_API_URL: string;
  readonly VITE_DEFAULT_MODEL: string;
  readonly VITE_ENABLE_AI_FEATURES: string;
  readonly VITE_ENABLE_TELEMETRY: string;
  readonly VITE_DEBUG_MODE: string;
  readonly MODE: string;
  readonly DEV: boolean;
  readonly PROD: boolean;
  readonly SSR: boolean;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

// Tauri API types
declare module '@tauri-apps/api/core' {
  export function invoke<T>(cmd: string, args?: any): Promise<T>;
}

declare module '@tauri-apps/api/event' {
  export interface Event<T> {
    event: string;
    windowLabel: string;
    id: number;
    payload: T;
  }

  export function listen<T>(
    event: string,
    handler: (event: Event<T>) => void
  ): Promise<() => void>;

  export function emit(event: string, payload?: any): Promise<void>;
}

declare module '@tauri-apps/api/path' {
  export function appDataDir(): Promise<string>;
  export function homeDir(): Promise<string>;
  export function configDir(): Promise<string>;
}

declare module '@tauri-apps/api/fs' {
  export function readTextFile(path: string): Promise<string>;
  export function writeTextFile(path: string, data: string): Promise<void>;
}

// XTerm.js types
declare module 'xterm' {
  export interface ITheme {
    background?: string;
    foreground?: string;
    cursor?: string;
    cursorAccent?: string;
    selectionBackground?: string;
    black?: string;
    red?: string;
    green?: string;
    yellow?: string;
    blue?: string;
    magenta?: string;
    cyan?: string;
    white?: string;
    brightBlack?: string;
    brightRed?: string;
    brightGreen?: string;
    brightYellow?: string;
    brightBlue?: string;
    brightMagenta?: string;
    brightCyan?: string;
    brightWhite?: string;
  }

  export interface ITerminalAddon {
    activate(terminal: Terminal): void;
    dispose(): void;
  }

  export class Terminal {
    constructor(options?: { theme?: ITheme; fontFamily?: string; fontSize?: number; fontWeight?: string; lineHeight?: number; letterSpacing?: number; cursorBlink?: boolean; cursorStyle?: string; scrollback?: number; tabStopWidth?: number; });
    open(element: HTMLElement): void;
    write(data: string): void;
    writeln(data: string): void;
    clear(): void;
    dispose(): void;
    onData(callback: (data: string) => void): void;
    onKey(callback: (key: { key: string; domEvent: KeyboardEvent }) => void): void;
    loadAddon(addon: ITerminalAddon): void;
    focus(): void;
    blur(): void;
    scrollToBottom(): void;
    resize(cols: number, rows: number): void;
    rows: number;
    cols: number;
  }
}

declare module 'xterm-addon-fit' {
  import { ITerminalAddon } from 'xterm';
  export class FitAddon implements ITerminalAddon {
    activate(terminal: any): void;
    dispose(): void;
    fit(): void;
    proposeDimensions(): { cols: number; rows: number } | undefined;
  }
}

declare module 'xterm-addon-web-links' {
  import { ITerminalAddon } from 'xterm';
  export class WebLinksAddon implements ITerminalAddon {
    constructor();
    activate(terminal: any): void;
    dispose(): void;
  }
}

declare module 'xterm-addon-search' {
  import { ITerminalAddon } from 'xterm';
  export class SearchAddon implements ITerminalAddon {
    constructor();
    activate(terminal: any): void;
    dispose(): void;
    findNext(term: string): boolean;
    findPrevious(term: string): boolean;
  }
}

declare module 'xterm-addon-canvas' {
  import { ITerminalAddon } from 'xterm';
  export class CanvasAddon implements ITerminalAddon {
    constructor();
    activate(terminal: any): void;
    dispose(): void;
  }
}

export {};
