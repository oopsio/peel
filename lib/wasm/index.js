/*
 * Copyright 2026 The Peel Authors. All rights reserved.
 * This file is licensed under the MIT License.
 */

"use strict";

(() => {
  const _encoder = new TextEncoder("utf-8");
  const decoder = new TextDecoder("utf-8");

  globalThis.Peel = class {
    constructor() {
      this.argv = ["peel"];
      this.env = {};
      this.mem = null;
      this._inst = null;

      this.importObject = {
        env: {
          wasm_write: (ptr, len) => {
            const buf = new Uint8Array(this._inst.exports.memory.buffer, ptr, len);
            console.log(decoder.decode(buf));
          },

          wasm_now: () => performance.now(),

          wasm_get_random: (ptr, len) => {
            const buf = new Uint8Array(this._inst.exports.memory.buffer, ptr, len);
            crypto.getRandomValues(buf);
          },

          wasm_panic: (ptr, len) => {
            const buf = new Uint8Array(this._inst.exports.memory.buffer, ptr, len);
            throw new Error("Peel Runtime Panic: " + decoder.decode(buf));
          },
        },
      };
    }

    async run(wasmSource) {
      const result = await WebAssembly.instantiate(wasmSource, this.importObject);
      this._inst = result.instance;
      this.mem = new DataView(this._inst.exports.memory.buffer);
      try {
        return this._inst.exports.main();
      } catch (e) {
        console.error("Peel Execution Error:", e);
      }
    }
  };
})();
