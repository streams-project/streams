// Minimal client for headless Chrome over the DevTools protocol. Uses the
// WebSocket built into Node, so no dependency is needed.

import { spawn } from "node:child_process";
import type { ChildProcess } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

interface Pending { resolve(v: unknown): void; reject(e: Error): void; }

export class Chrome {
  process: ChildProcess;
  socket: WebSocket;
  profile: string;
  session = "";
  nextId = 1;
  pending = new Map<number, Pending>();

  constructor(process: ChildProcess, socket: WebSocket, profile: string) {
    this.process = process;
    this.socket = socket;
    this.profile = profile;
    socket.onmessage = (event) => {
      const m = JSON.parse(String(event.data)) as { id?: number; result?: unknown; error?: { message: string } };
      if (m.id === undefined) return;
      const p = this.pending.get(m.id);
      if (!p) return;
      this.pending.delete(m.id);
      if (m.error) p.reject(new Error(m.error.message));
      else p.resolve(m.result);
    };
  }

  send<T = unknown>(method: string, params: object = {}, session = this.session): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.socket.send(JSON.stringify({ id, method, params, sessionId: session || undefined }));
    });
  }

  // Runs an expression in the page. The value must be JSON safe.
  async evaluate<T = unknown>(expression: string): Promise<T> {
    const r = await this.send<{ result: { value: T }; exceptionDetails?: { text: string; exception?: { description?: string } } }>("Runtime.evaluate", {
      expression, awaitPromise: true, returnByValue: true
    });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value;
  }

  async open(url: string, width: number, height: number): Promise<void> {
    const target = await this.send<{ targetId: string }>("Target.createTarget", { url: "about:blank" }, "");
    const attached = await this.send<{ sessionId: string }>("Target.attachToTarget", { targetId: target.targetId, flatten: true }, "");
    this.session = attached.sessionId;
    await this.send("Page.enable");
    await this.send("Runtime.enable");
    await this.send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false });
    await this.send("Emulation.setFocusEmulationEnabled", { enabled: true });
    await this.send("Page.navigate", { url });
  }

  async close(): Promise<void> {
    try { await this.send("Browser.close", {}, ""); } catch { /* already gone */ }
    this.socket.close();
    this.process.kill();
    await new Promise((r) => setTimeout(r, 300));
    rmSync(this.profile, { recursive: true, force: true });
  }
}

export async function launchChrome(path: string): Promise<Chrome> {
  const profile = mkdtempSync(join(tmpdir(), "tv-chrome-"));
  const child = spawn(path, [
    "--headless=new", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--no-first-run",
    "--no-default-browser-check", "--use-angle=metal", "--ignore-gpu-blocklist", "--enable-gpu-rasterization",
    "--disable-background-timer-throttling", "--disable-renderer-backgrounding",
    "--disable-backgrounding-occluded-windows", "about:blank"
  ], { stdio: ["ignore", "ignore", "pipe"] });
  const url = await new Promise<string>((resolve, reject) => {
    let log = "";
    const timer = setTimeout(() => reject(new Error("Chrome did not start: " + log)), 20000);
    child.stderr!.on("data", (d: Buffer) => {
      log += d.toString();
      const m = /DevTools listening on (ws:\S+)/.exec(log);
      if (m) { clearTimeout(timer); resolve(m[1]); }
    });
    child.on("exit", (code) => reject(new Error(`Chrome exited with ${code}: ${log}`)));
  });
  const socket = new WebSocket(url);
  await new Promise<void>((resolve, reject) => {
    socket.onopen = () => resolve();
    socket.onerror = () => reject(new Error("could not reach Chrome"));
  });
  return new Chrome(child, socket, profile);
}
