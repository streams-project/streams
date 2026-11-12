// Reading a server's answer into the type it is.
//
// A `.ts` file for the reason `effects.ts` gives at length: ArkTS bans `any`
// (`arkts-no-any-unknown`, 10605008) and `JSON.parse` is typed `any`, so the official ArkTS reading
// is `let held: Record<string, Object> = JSON.parse(text)` followed by a cast per field -- which for
// a nested `Screen` means walking the whole tree by hand. The rules are scoped to `.ets` files, so
// the read happens here, once, in TypeScript, and the `.ets` side gets a typed object or null.
//
// Nothing here validates. The core has already decided whether a body is good; what this does is
// refuse to throw on one that is not JSON, because a screen that threw on a malformed body would
// take the app down over a proxy's error page.
import {
  Hello, More, Nav, NotificationCount, ProfileGate, Screen, Season, Strings, TitleDetail,
} from './contract';

function read<T>(text: string): T | null {
  try {
    return JSON.parse(text) as T;
  } catch {
    return null;
  }
}

/** True for a status the server means as an answer. */
export function good(status: number): boolean {
  return status >= 200 && status <= 299;
}

export function readScreen(text: string): Screen | null {
  return read<Screen>(text);
}

export function readMore(text: string): More | null {
  return read<More>(text);
}

export function readNav(text: string): Nav | null {
  return read<Nav>(text);
}

export function readTitle(text: string): TitleDetail | null {
  return read<TitleDetail>(text);
}

export function readSeason(text: string): Season | null {
  return read<Season>(text);
}

export function readStrings(text: string): Strings | null {
  return read<Strings>(text);
}

export function readNotificationCount(text: string): NotificationCount | null {
  return read<NotificationCount>(text);
}

export function readHello(text: string): Hello | null {
  return read<Hello>(text);
}

export function readProfileGate(text: string): ProfileGate | null {
  return read<ProfileGate>(text);
}
