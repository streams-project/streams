// The host of a session in a browser. It does what templates/terminal does, with the same tape:
// it holds the clock and the one wake, performs effects, and writes every message.

import type { Core, CoreModule } from "../../../bridges/ts/core";

interface Effect {
  id: string;
  kind: string;
  op?: string;
  key?: string;
  name?: string;
  value?: string;
  method?: string;
  path?: string;
}

interface Out {
  effects: Effect[];
  next_wake_ms?: number;
}

/** What one tape left behind. */
export interface Run {
  transcript: string;
  steps: number;
}

class Host {
  private core: Core | null = null;
  private readonly store = new Map<string, string>();
  private readonly secrets = new Map<string, string>();
  private hasStore = false;
  private hasSecret = false;
  private contract = "";
  private wallStart = 0;
  private clock = 0;
  private monoBase = 0;
  private wake: number | null = null;
  private nextAnswer = 0;
  private readonly lines: string[];
  readonly run: Run = { transcript: "", steps: 0 };

  constructor(
    private readonly module: CoreModule,
    tape: string,
  ) {
    this.lines = tape.split("\n").filter((line) => line.length > 0);
  }

  play(): void {
    for (const line of this.lines) {
      const space = line.indexOf(" ");
      const word = space < 0 ? line : line.slice(0, space);
      const rest = line.slice(space + 1);
      if (word === "contract") {
        this.contract = rest;
      } else if (word === "wall") {
        this.wallStart = Number(rest);
      } else if (word === "device") {
        this.hasStore = rest[0] === "1";
        this.hasSecret = rest[2] === "1";
      } else if (word === "open") {
        this.open(rest);
      } else if (word === "at") {
        this.advanceTo(Number(rest));
      } else if (word === "forget") {
        this.secrets.clear();
      } else if (word === "step") {
        this.step(rest);
      } else if (!line.startsWith("http\t")) {
        throw new Error(`unknown line: ${line}`);
      }
    }
    this.core?.close();
  }

  private open(line: string): void {
    const config = line.slice(2);
    if (line[0] === "1") {
      this.monoBase = this.clock;
    }
    this.core?.close();
    this.core = this.module.open(config);
    this.wake = null;
    this.run.transcript += `open ${config}\n`;
  }

  private advanceTo(at: number): void {
    while (this.wake !== null && this.wake + this.monoBase <= at) {
      this.clock = this.wake + this.monoBase;
      this.step("[]");
    }
    this.clock = at;
  }

  private step(firstEvents: string): void {
    let events = firstEvents;
    for (;;) {
      const mono = this.clock - this.monoBase;
      const wall = this.wallStart + this.clock;
      const input = `{"contract":"${this.contract}","events":${events},"mono_ms":${mono},"now_ms":${wall},"type":"core_in"}`;
      const output = this.core!.step(input);
      this.run.steps += 1;
      this.run.transcript += `in ${input}\nout ${output}\n`;
      const out = JSON.parse(output) as Out;
      this.wake = out.next_wake_ms ?? null;
      const results: string[] = [];
      for (const effect of out.effects) {
        this.perform(effect, results);
      }
      if (results.length === 0) {
        break;
      }
      events = `[${results.join(",")}]`;
    }
    if (this.wake !== null && this.wake + this.monoBase <= this.clock) {
      throw new Error("busy loop");
    }
  }

  private perform(effect: Effect, results: string[]): void {
    const id = JSON.stringify(effect.id);
    if (effect.kind === "http") {
      results.push(this.answer(effect));
    } else if (effect.kind === "store" && this.hasStore) {
      results.push(this.table(this.store, id, effect, effect.key, "store_done"));
    } else if (effect.kind === "secret" && this.hasSecret) {
      results.push(this.table(this.secrets, id, effect, effect.name, "secret_done"));
    } else if (!["player", "view", "open", "log"].includes(effect.kind)) {
      results.push(`{"id":${id},"type":"unsupported"}`);
    }
  }

  private table(
    table: Map<string, string>,
    id: string,
    effect: Effect,
    key: string = "",
    done: string,
  ): string {
    let keys = "";
    let value = "";
    if (effect.op === "list") {
      const names = [...table.keys()].filter((name) => name.startsWith(key)).sort();
      keys = `,"keys":${JSON.stringify(names)}`;
    } else if (effect.op === "get") {
      const held = table.get(key);
      value = held === undefined ? "" : `,"value":${JSON.stringify(held)}`;
    } else if (effect.op === "put") {
      table.set(key, effect.value ?? "");
    } else if (effect.op === "delete") {
      table.delete(key);
    }
    return `{"id":${id}${keys},"type":"${done}"${value}}`;
  }

  /** The next answer on the tape, which must be the answer to this effect. */
  private answer(effect: Effect): string {
    while (this.nextAnswer < this.lines.length && !this.lines[this.nextAnswer].startsWith("http\t")) {
      this.nextAnswer += 1;
    }
    const line = this.lines[this.nextAnswer];
    this.nextAnswer += 1;
    if (line === undefined) {
      throw new Error(`no answer left for ${effect.id}`);
    }
    const [, id, method, path, ...event] = line.split("\t");
    const asked = [effect.id, effect.method, effect.path].map((part) => JSON.stringify(part));
    if (id !== asked[0] || method !== asked[1] || path !== asked[2]) {
      throw new Error(`the tape does not fit ${asked.join(" ")}`);
    }
    return event.join("\t");
  }
}

/** Run one tape through a module. */
export function playTape(module: CoreModule, tape: string): Run {
  const host = new Host(module, tape);
  host.play();
  return host.run;
}
