// Optional WASM accelerator for NXS reducers.
// Works in both Node.js and browsers (ES modules).
//
// Usage (browser):
//   const wasm = await loadWasm("./wasm/nxs_reducers.wasm");
//   const buf  = new Uint8Array(await (await fetch("data.nxb")).arrayBuffer());
//   const r    = new NxsReader(buf);
//   r.useWasm(wasm);          // copies bytes into WASM memory
//   r.sumF64("score");
//
// Usage (Node.js):
//   import { loadWasm, readNxbIntoWasm } from "./wasm.js";
//   const wasm = await loadWasm();                     // default path resolved via import.meta.url
//   const buf  = readNxbIntoWasm(wasm, "data.nxb");    // zero-copy fast path
//   const r    = new NxsReader(buf);
//   r.useWasm(wasm);                                    // no-op (already resident)
//   r.sumF64("score");

export class NxsWasm {
  constructor(instance, memory, dataBase) {
    this.instance = instance;
    this.memory = memory;
    this.dataBase = dataBase;
    this.fns = instance.exports;
    this.bytes = new Uint8Array(memory.buffer);
    this.loadedBytes = 0;
  }

  allocBuffer(n) {
    this._ensureCapacity(n);
    this.loadedBytes = n;
    return new Uint8Array(this.memory.buffer, this.dataBase, n);
  }

  _ensureCapacity(n) {
    const end = this.dataBase + n;
    const have = this.memory.buffer.byteLength;
    if (end > have) {
      const extraPages = Math.ceil((end - have) / 65536);
      this.memory.grow(extraPages);
    }
    this.bytes = new Uint8Array(this.memory.buffer);
  }

  loadPayload(nxbBytes) {
    if (this._sharesMemory(nxbBytes)) {
      this.loadedBytes = nxbBytes.byteLength;
      return;
    }
    this._ensureCapacity(nxbBytes.byteLength);
    this.bytes.set(nxbBytes, this.dataBase);
    this.loadedBytes = nxbBytes.byteLength;
  }

  _sharesMemory(nxbBytes) {
    return nxbBytes.buffer === this.memory.buffer
        && nxbBytes.byteOffset === this.dataBase;
  }
}

/**
 * Load the WASM module. Works in Node.js or browsers.
 *
 * @param {string | URL} [wasmUrl] — URL or path to nxs_reducers.wasm. When
 *   running in Node with no argument, resolves relative to this module.
 *   When running in a browser, defaults to "./wasm/nxs_reducers.wasm".
 * @param {object} [opts]
 * @param {number} [opts.initialPages=1024] — initial memory pages (64 KB each)
 */
export async function loadWasm(wasmUrl, opts = {}) {
  const initialPages = opts.initialPages ?? 1024;
  const memory = new WebAssembly.Memory({ initial: initialPages, maximum: 65536 });

  const wasmBytes = await _fetchWasmBytes(wasmUrl);
  const mod = await WebAssembly.instantiate(wasmBytes, { env: { memory } });

  return new NxsWasm(mod.instance, memory, 65536);
}

async function _fetchWasmBytes(wasmUrl) {
  // Browser: use fetch when available and we have a URL.
  if (typeof fetch !== "undefined" && wasmUrl !== undefined) {
    const res = await fetch(wasmUrl);
    if (!res.ok) throw new Error(`failed to load wasm: ${res.status}`);
    const buf = await res.arrayBuffer();
    return new Uint8Array(buf);
  }

  // Node: read from disk. Dynamic import so the browser bundler doesn't try
  // to resolve node:fs at parse time.
  const { readFileSync } = await import("node:fs");
  if (wasmUrl === undefined) {
    const { fileURLToPath } = await import("node:url");
    const { dirname, join } = await import("node:path");
    const here = dirname(fileURLToPath(import.meta.url));
    wasmUrl = join(here, "wasm/nxs_reducers.wasm");
  }
  return readFileSync(wasmUrl);
}

/**
 * Node-only convenience: open an .nxb file and read it directly into WASM
 * memory. Returns a Uint8Array view suitable for `new NxsReader(...)`.
 *
 * Throws in browsers (use `fetch` + `arrayBuffer` + `new NxsReader` there).
 */
export async function readNxbIntoWasm(wasm, path) {
  if (typeof process === "undefined" || !process.versions?.node) {
    throw new Error("readNxbIntoWasm is Node-only; use fetch() in browsers");
  }
  const { openSync, fstatSync, readSync, closeSync } = await import("node:fs");
  const fd = openSync(path, "r");
  try {
    const size = fstatSync(fd).size;
    const buf = wasm.allocBuffer(size);
    readSync(fd, buf, 0, size, 0);
    return buf;
  } finally {
    closeSync(fd);
  }
}
