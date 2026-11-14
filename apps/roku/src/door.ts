// Where this app's server comes from, and which addresses it will take.
//
// Everyone runs their own server, so nothing is built in and every family has to answer this. The
// three that have a host answer it three ways, and the Roku answer is a fourth:
//
//   | family  | where the address comes from                                                   |
//   |---------|--------------------------------------------------------------------------------|
//   | web     | nowhere: a browser is already at the server it loaded the page from            |
//   | android | a kept value, else a screen of its own with a text field (`door/ServerAddress.kt`) |
//   | apple   | a kept value, else the `SERVER_URL` environment variable, else that same screen (`CoreHost/Host.swift:127-131`) |
//   | roku    | the parameters the channel was launched with, else a kept value                |
//
// **The decision, and why it is this one.** A Roku app built with RSG 2.0 cannot be installed from
// the Streaming Store before Fall 2027 (README section 1): the only way onto a device is `rk dev`,
// run by a person at a computer. That person is already typing a command, and a television remote
// has no keyboard. So the address is a parameter of the launch, kept in the registry on the way
// past, and from the second run onwards the television starts straight into the app.
//
// What that buys, beyond not writing an on-screen keyboard: it is the **one** address screen this
// folder would have had which cannot be drawn in the viewer's language, because the words come
// from `/v1/strings/{lang}` on the server that has not been named yet. Android's own address
// screen holds four English literals for exactly that reason (`door/ServerAddress.kt:57,70,74`),
// and Apple's holds the same. This folder holds no English and now needs none.
//
// What it costs, stated rather than hidden: a viewer who is handed a device cannot move it to
// another server without a computer. The day the store opens, this family needs the screen the
// other two have, and it needs string keys that can be drawn before any string has arrived.

/** The name of the launch parameter that carries the address. This surface's own, not Roku's. */
export const SERVER_PARAMETER = "server";

/**
 * Where the kept address lives. The same name Android and Apple use (`core/Host.kt:88`,
 * `CoreHost/Host.swift:37`), so a person reading one family recognises it in the next.
 */
export const SERVER_KEY = "app:server";

/** Which of the two places an address came from, or neither. */
export type Source = "launch" | "kept" | "none";

export interface Given {
  server: string;
  from: Source;
}

/**
 * The address this run starts with.
 *
 * The launch parameter wins over the kept one, and deliberately: it is the only way a person with
 * a computer can move a television that is already set up, and a kept value that outranked it
 * would make a device impossible to re-point without clearing the registry.
 */
export function serverFrom(given: Record<string, string>, kept: string | null): Given {
  const launched = (given[SERVER_PARAMETER] ?? "").trim();
  if (launched !== "") return { server: launched, from: "launch" };
  const held = (kept ?? "").trim();
  if (held !== "") return { server: held, from: "kept" };
  return { server: "", from: "none" };
}

/** An address with the spaces and the trailing slash off it. (`core/Host.kt:170`) */
export function tidy(server: string): string {
  return server.trim().replace(/\/+$/, "");
}

/**
 * True when this address is reached in the clear, false when it is encrypted, and **null when it
 * may not be used at all**. A mirror of `core/Host.kt:101-132` and `CoreHost/Host.swift:172-200`,
 * branch for branch.
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

/** The host of an address, with the port, the path and an IPv6 address's brackets off. (`Host.kt:109-113`) */
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
 * The viewer's own machine or their own network: a hop no stranger is on. Names are here because a
 * server is usually found by its name on the network, not by its number. (`Host.kt:119-132`)
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
