// Loads core.wasm with the plain WebAssembly API and carries text across the six exports. No
// rules, no generated glue and no dependency. It takes bytes, so the host decides where they come
// from. Pointers and lengths are i32, so this runs on engines with no i64 across the boundary.

const ABI = 1;
const OK = 0;
const BAD_MESSAGE = 1;

interface Exports {
  memory: WebAssembly.Memory;
  core_abi(): number;
  core_alloc(len: number): number;
  core_free(ptr: number, len: number): void;
  core_open(cfg_ptr: number, cfg_len: number): number;
  core_step(
    handle: number,
    in_ptr: number,
    in_len: number,
    out_ptr_out: number,
    out_len_out: number,
  ): number;
  core_close(handle: number): void;
}

/** A call was refused and the core is unchanged. */
export class CoreRefused extends Error {}

/** The module trapped, so nothing in it can be trusted. Load it again and start from what was stored. */
export class CoreTrapped extends Error {}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** One loaded module. It holds any number of cores, all used from one thread. */
export class CoreModule {
  private trapped = false;

  private constructor(private readonly exports: Exports) {}

  /** Load from the bytes of core.wasm. The module imports nothing. */
  static async load(bytes: BufferSource): Promise<CoreModule> {
    const { instance } = await WebAssembly.instantiate(bytes, {});
    const module = new CoreModule(instance.exports as unknown as Exports);
    if (module.exports.core_abi() !== ABI) {
      throw new CoreRefused("this loader does not fit this module");
    }
    return module;
  }

  /** Bytes of linear memory in use, which only grows. */
  get memoryBytes(): number {
    return this.exports.memory.buffer.byteLength;
  }

  /** Open a core from a config, with a `layer` key when the host wants the offline layer. */
  open(config: string): Core {
    const handle = this.guarded(() => {
      const [at, len] = this.place(config);
      const opened = this.exports.core_open(at, len);
      this.exports.core_free(at, len);
      return opened;
    });
    if (handle === 0) {
      throw new CoreRefused("the config does not read");
    }
    return new Core(this, handle);
  }

  /** @internal Every call into the module goes through here, so a trap is seen once and kept. */
  guarded<T>(work: () => T): T {
    if (this.trapped) {
      throw new CoreTrapped("the module trapped earlier");
    }
    try {
      return work();
    } catch (error) {
      // A stack that ran out shows as a RangeError, and leaves the module as unsound as a trap.
      if (error instanceof WebAssembly.RuntimeError || error instanceof RangeError) {
        this.trapped = true;
        throw new CoreTrapped(error.message);
      }
      throw error;
    }
  }

  /** @internal One step, with `handle` from `open`. */
  step(handle: number, input: string): string {
    return this.guarded(() => {
      const [at, len] = this.place(input);
      const slots = this.exports.core_alloc(8);
      const status = this.exports.core_step(handle, at, len, slots, slots + 4);
      // Read memory only now: a step can grow it, which replaces the buffer.
      const view = new DataView(this.exports.memory.buffer);
      const outAt = view.getUint32(slots, true);
      const outLen = view.getUint32(slots + 4, true);
      this.exports.core_free(slots, 8);
      this.exports.core_free(at, len);
      if (status !== OK) {
        const why = status === BAD_MESSAGE ? "the message does not read" : `status ${status}`;
        throw new CoreRefused(why);
      }
      const bytes = new Uint8Array(this.exports.memory.buffer, outAt, outLen);
      const text = decoder.decode(bytes);
      this.exports.core_free(outAt, outLen);
      return text;
    });
  }

  /** @internal */
  close(handle: number): void {
    this.guarded(() => this.exports.core_close(handle));
  }

  private place(text: string): [number, number] {
    const bytes = encoder.encode(text);
    const at = this.exports.core_alloc(bytes.length);
    if (at === 0) {
      throw new CoreRefused("no memory left");
    }
    new Uint8Array(this.exports.memory.buffer, at, bytes.length).set(bytes);
    return [at, bytes.length];
  }
}

/** One running core. */
export class Core {
  /** @internal */
  constructor(
    private readonly module: CoreModule,
    private readonly handle: number,
  ) {}

  /** One step of JSON text. */
  step(input: string): string {
    return this.module.step(this.handle, input);
  }

  close(): void {
    this.module.close(this.handle);
  }
}
