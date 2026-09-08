/**
 * SC-Substrate Universal Deterministic WebAssembly Client.
 * Zero-copy memory mapped evaluation substrate for Node.js, Bun, Deno, and Web Browsers.
 */

export const PREAMBLE_CANARY = new Uint8Array([0x53, 0x43, 0x43, 0x41, 0x53, 0x57, 0x41, 0x53]); // SCCASWAS
export const TRAILING_SEAL_CANARY = new Uint8Array([0x53, 0x43, 0x53, 0x5f, 0x53, 0x45, 0x41, 0x4c]); // SCS_SEAL

export enum OpCode {
  EvalSymbolic = 1,
  IntegrateMonodromy = 2,
  DixonSolve = 3,
  DeepPolyVerify = 4,
  ArithmeticSimplify = 5,
  CliffordMultivector = 6,
  TensorSlice = 7,
}

export interface EvaluationResult {
  statusCode: number;
  payload: string;
  attestationDigestHex: string;
}

export class SCCASWasmClient {
  private instance: WebAssembly.Instance;
  private memory: WebAssembly.Memory;

  constructor(instance: WebAssembly.Instance) {
    this.instance = instance;
    this.memory = (instance.exports.memory as WebAssembly.Memory);
  }

  public static async createFromBytes(wasmBytes: BufferSource): Promise<SCCASWasmClient> {
    const module = await WebAssembly.compile(wasmBytes);
    const instance = await WebAssembly.instantiate(module, {});
    return new SCCASWasmClient(instance);
  }

  public getVersion(): number {
    return (this.instance.exports.sc_cas_wasm_version as Function)();
  }

  public execute(opcode: OpCode, flags: number, payload: string): EvaluationResult {
    const encoder = new TextEncoder();
    const payloadBytes = encoder.encode(payload);

    // Build canonical wire request
    const totalReqLen = 20 + payloadBytes.length;
    const reqBuffer = new Uint8Array(totalReqLen);
    reqBuffer.set(PREAMBLE_CANARY, 0);

    const view = new DataView(reqBuffer.buffer);
    view.setUint32(8, opcode, true);
    view.setUint32(12, flags, true);
    view.setUint32(16, payloadBytes.length, true);
    reqBuffer.set(payloadBytes, 20);

    // Allocate input buffer inside Wasm linear memory
    const allocFn = this.instance.exports.sc_cas_wasm_alloc as Function;
    const freeFn = this.instance.exports.sc_cas_wasm_free as Function;
    const dispatchFn = this.instance.exports.sc_cas_wasm_dispatch as Function;

    const inOffset = allocFn(totalReqLen);
    if (inOffset === 0) {
      throw new Error("Failed to allocate linear memory for request");
    }

    const memoryView = new Uint8Array(this.memory.buffer);
    memoryView.set(reqBuffer, inOffset);

    // Allocate output buffer (64 KiB)
    const outCapacity = 65536;
    const outOffset = allocFn(outCapacity);
    if (outOffset === 0) {
      freeFn(inOffset, totalReqLen);
      throw new Error("Failed to allocate linear memory for response");
    }

    const written = dispatchFn(inOffset, totalReqLen, outOffset, outCapacity);
    if (written === 0) {
      freeFn(inOffset, totalReqLen);
      freeFn(outOffset, outCapacity);
      throw new Error("Wasm dispatch failed or boundary invariant violated");
    }

    // Read response slice from Wasm linear memory
    const respSlice = new Uint8Array(this.memory.buffer, outOffset, written).slice();

    // Clean up allocated linear memory blocks
    freeFn(inOffset, totalReqLen);
    freeFn(outOffset, outCapacity);

    // Parse response envelope
    const respView = new DataView(respSlice.buffer);
    const statusCode = respView.getUint32(8, true);
    const respPayloadLen = respView.getUint32(12, true);
    const respPayloadBytes = respSlice.subarray(16, 16 + respPayloadLen);
    const digestBytes = respSlice.subarray(16 + respPayloadLen, 16 + respPayloadLen + 32);

    const decoder = new TextDecoder();
    const resultString = decoder.decode(respPayloadBytes);
    const digestHex = Array.from(digestBytes).map(b => b.toString(16).padStart(2, '0')).join('');

    return {
      statusCode,
      payload: resultString,
      attestationDigestHex: digestHex,
    };
  }
}
