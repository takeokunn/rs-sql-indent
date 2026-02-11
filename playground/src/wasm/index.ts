import init, { format_sql_wasm } from '../../pkg/rs_sql_indent.js';
import wasmUrl from '../../pkg/rs_sql_indent_bg.wasm?url';

let wasmReady = false;

export async function initWasm(): Promise<void> {
  await init({ module_or_path: wasmUrl });
  wasmReady = true;
}

export function formatSql(input: string, uppercase: boolean, style: string): string {
  if (!wasmReady) {
    throw new Error('WASM module not initialized');
  }
  return format_sql_wasm(input, uppercase, style);
}
