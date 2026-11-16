// Where this app's server comes from, and which addresses it will take.
//
// Everyone runs their own server, so every family resolves its address from its build or launch
// without asking the viewer to type one on an address screen:
//
//   | family  | where the address comes from                                                   |
//   |---------|--------------------------------------------------------------------------------|
//   | web     | nowhere: a browser is already at the server it loaded the page from            |
//   | native  | bound into the build from `deployment.json` or `--server` (`launcher/src/address.rs`) |
//   | roku    | the parameters the channel was launched with, else a kept value, else the bound build address |
//
// **Why launch parameters outrank kept and bound.** An RSG 2.0 app is installed with `rk dev`,
// run by a person at a computer, and a television remote has no keyboard. Passing `server` as a
// launch parameter lets that person re-point an already built or already installed channel without
// clearing its registry; when none is passed, a kept value or the bound build address still starts
// straight into the app.

/** The name of the launch parameter that carries the address. This surface's own, not Roku's. */
export const SERVER_PARAMETER = "server";

/**
 * Where the kept address lives in the device store.
 */
export const SERVER_KEY = "app:server";

/** Which of the three places an address came from, or none. */
export type Source = "launch" | "kept" | "bound" | "none";

export interface Given {
  server: string;
  from: Source;
}

/**
 * The address this run starts with.
 *
 * The launch parameter wins over the kept one, which wins over the build's bound address: a kept
 * value that outranked the launch parameter would make a device impossible to re-point without
 * clearing the registry, and a bound build address gives a fresh device a server when neither was
 * passed at launch.
 */
export function serverFrom(given: Record<string, string>, kept: string | null, bound = ""): Given {
  const launched = (given[SERVER_PARAMETER] ?? "").trim();
  if (launched !== "") return { server: launched, from: "launch" };
  const held = (kept ?? "").trim();
  if (held !== "") return { server: held, from: "kept" };
  const built = bound.trim();
  if (built !== "") return { server: built, from: "bound" };
  return { server: "", from: "none" };
}

/** An address with the spaces and the trailing slash off it. (`launcher/src/address.rs`) */
export function tidy(server: string): string {
  return server.trim().replace(/\/+$/, "");
}

/**
 * True when this address is reached in the clear, false when it is encrypted, and **null when it
 * may not be used at all**. Matches `launcher/src/address.rs` (`check_bind` and `own_network`).
 *
 * The rule, in one sentence, because the shape of the answer hides it: plain http carries the
 * session's bearer and refresh tokens where anyone on the hop can read them, so it is allowed only
 * to the viewer's own machine or their own network, and everything else must be https.
 */
export function encryptionOf(server: string): boolean | null {
  const lower = tidy(server).toLowerCase();
  if (lower.startsWith("https://")) return false;
  if (!lower.startsWith("http://")) return null;
  return ownNetwork(hostOf(lower)) ? true : null;
}

/** The host of an address, with the port, the path and an IPv6 address's brackets off. */
export function hostOf(address: string): string {
  const after = upTo(address.slice("http://".length), "/");
  if (after.startsWith("[")) return upTo(after.slice(1), "]");
  return upTo(after, ":");
}

function upTo(text: string, mark: string): string {
  const at = text.indexOf(mark);
  return at < 0 ? text : text.slice(0, at);
}

/**
 * The viewer's own machine or their own network: a hop no stranger is on. Matches
 * `launcher/src/address.rs:own_network`.
 */
export function ownNetwork(host: string): boolean {
  if (host === "localhost" || host.endsWith(".local") || host === "::1") return true;
  const parts = host.split(".");
  if (parts.length !== 4) return false;
  const numbers: number[] = [];
  for (const part of parts) {
    if (!/^[0-9]+$/.test(part)) return false;
    numbers.push(Number(part));
  }
  if (numbers.some((one) => one < 0 || one > 255)) return false;
  const first = numbers[0] ?? -1;
  const second = numbers[1] ?? -1;
  if (first === 127 || first === 10) return true;
  if (first === 192 && second === 168) return true;
  if (first === 169 && second === 254) return true;
  return first === 172 && second >= 16 && second <= 31;
}
