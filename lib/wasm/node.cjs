/*
 * Copyright 2026 The Peel Authors. All rights reserved.
 * This file is licensed under the MIT License.
 */

"use strict";

const fs = require("fs");
const crypto = require("crypto");
const { performance } = require("perf_hooks");

class Peel {
  constructor() {
    this.argv = process.argv;
    this.env = process.env;
    this.mem = null;
    this._inst = null;

    const decoder = new TextDecoder("utf-8");

    this.importObject = {
      env: {
        wasm_write: (ptr, len) => {
          const buf = new Uint8Array(this._inst.exports.memory.buffer, ptr, len);
          process.stdout.write(decoder.decode(buf));
        },

        wasm_now: () => performance.now(),

        wasm_get_random: (ptr, len) => {
          const buf = new Uint8Array(this._inst.exports.memory.buffer, ptr, len);
          crypto.randomFillSync(buf);
        },

        wasm_panic: (ptr, len) => {
          const buf = new Uint8Array(this._inst.exports.memory.buffer, ptr, len);
          const msg = decoder.decode(buf);
          console.error(`Peel Runtime Panic: ${msg}`);
          process.exit(1);
        },
      },
    };
  }

  async run(wasmBuffer) {
    const result = await WebAssembly.instantiate(wasmBuffer, this.importObject);
    this._inst = result.instance;
    this.mem = new DataView(this._inst.exports.memory.buffer);

    try {
      return this._inst.exports.main();
    } catch (e) {
      console.error("Peel Execution Error:", e);
      throw e;
    }
  }
}

module.exports = Peel;

if (require.main === module) {
  (async () => {
    const peel = new Peel();
    const wasmPath = process.argv[2];
    if (!wasmPath) {
      console.error("Usage: node run_node.js <file.wasm>");
      process.exit(1);
    }
    const buffer = fs.readFileSync(wasmPath);
    await peel.run(buffer);
  })();
}
