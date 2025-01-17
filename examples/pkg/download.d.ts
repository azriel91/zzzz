/* tslint:disable */
/* eslint-disable */
export function wasm_init(url: string, name: string): Promise<WorkspaceAndOutput>;
export function wasm_fetch(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
export function wasm_status(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
export function wasm_goal(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
export function wasm_diff(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
export function wasm_ensure_dry(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
export function wasm_ensure(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
export function wasm_clean_dry(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
export function wasm_clean(workspace_and_output: WorkspaceAndOutput): Promise<WorkspaceAndOutput>;
/**
 * The `ReadableStreamType` enum.
 *
 * *This API requires the following crate features to be activated: `ReadableStreamType`*
 */
type ReadableStreamType = "bytes";
export class IntoUnderlyingByteSource {
  private constructor();
  free(): void;
  start(controller: ReadableByteStreamController): void;
  pull(controller: ReadableByteStreamController): Promise<any>;
  cancel(): void;
  readonly type: ReadableStreamType;
  readonly autoAllocateChunkSize: number;
}
export class IntoUnderlyingSink {
  private constructor();
  free(): void;
  write(chunk: any): Promise<any>;
  close(): Promise<any>;
  abort(reason: any): Promise<any>;
}
export class IntoUnderlyingSource {
  private constructor();
  free(): void;
  pull(controller: ReadableStreamDefaultController): Promise<any>;
  cancel(): void;
}
export class WorkspaceAndOutput {
  private constructor();
  free(): void;
  output: string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_workspaceandoutput_free: (a: number, b: number) => void;
  readonly __wbg_get_workspaceandoutput_output: (a: number) => [number, number];
  readonly __wbg_set_workspaceandoutput_output: (a: number, b: number, c: number) => void;
  readonly wasm_init: (a: number, b: number, c: number, d: number) => any;
  readonly wasm_fetch: (a: number) => any;
  readonly wasm_status: (a: number) => any;
  readonly wasm_goal: (a: number) => any;
  readonly wasm_diff: (a: number) => any;
  readonly wasm_ensure_dry: (a: number) => any;
  readonly wasm_ensure: (a: number) => any;
  readonly wasm_clean_dry: (a: number) => any;
  readonly wasm_clean: (a: number) => any;
  readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
  readonly intounderlyingsource_pull: (a: number, b: any) => any;
  readonly intounderlyingsource_cancel: (a: number) => void;
  readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
  readonly intounderlyingsink_write: (a: number, b: any) => any;
  readonly intounderlyingsink_close: (a: number) => any;
  readonly intounderlyingsink_abort: (a: number, b: any) => any;
  readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
  readonly intounderlyingbytesource_type: (a: number) => number;
  readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
  readonly intounderlyingbytesource_start: (a: number, b: any) => void;
  readonly intounderlyingbytesource_pull: (a: number, b: any) => any;
  readonly intounderlyingbytesource_cancel: (a: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_6: WebAssembly.Table;
  readonly closure2980_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure3194_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
