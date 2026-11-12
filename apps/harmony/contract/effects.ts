// The contract's unions, narrowed to the members a host reads.
//
// A `.ts` FILE ON PURPOSE, and this is the answer to OPEN QUESTION Q2.
//
// ArkTS is a narrower language than TypeScript, and three of the four constructs the generated
// contract uses are hard compile errors in it: an intersection with an empty object type
// (`arkts-no-intersection-types`, 10605019), an anonymous object type inside a union
// (`arkts-no-obj-literals-as-types`, 10605040) and a utility type such as `Extract`
// (`arkts-no-utility-types`, 10605138). The generated file uses the first forty times, the second
// eighty times, and carries three template literal types and one `as const` besides -- counted in
// `platform/contract/generated/types/contract.ts` on 2026-11-12.
//
// None of that has to change, because the rules are scoped by FILE EXTENSION. The migration
// cookbook's `arkts-no-ts-deps` (10605147) states the direction in so many words:
//
//     "An .ets file can import source code of an .ets/.ts/.js file, but a .ts/.js file cannot
//      import source code of an .ets file."
//
// and the background page scopes the check itself: "the SDK adds the ArkTS syntax validation **for
// .ets files** to the compilation process". So a `.ts` file in a HarmonyOS module is compiled as
// TypeScript -- the SDK of API 11 used TypeScript 4.9.5 with `target: es2017` -- and `.ets` files
// may import from it.
//
// This file is therefore the seam. Everything the contract does that ArkTS refuses stays on this
// side of it, named, and the `.ets` side imports ordinary named types. Three prices, all official
// and all paid knowingly:
//
//   1. "when data and objects of TS/JS are used as those of ArkTS, the static type check of ArkTS
//      may be bypassed, causing unexpected behavior or extra overhead". So this file names the
//      shapes it exports rather than widening anything, and the host reads only tagged members.
//   2. A `.ts` file can never import an `.ets` file (10605147), so this is a leaf and must stay one.
//   3. The ArkCompiler runtime prohibits circular dependency outright -- "Application loading
//      fails" -- so nothing here may ever reach back up.
//
// What this replaces: three `Extract<...>` aliases that sat in `core/Host.ets`, where they were
// illegal, and three casts between unrelated nominal types, which ArkTS would have refused because
// it has no structural typing (`arkts-no-structural-typing`, 10605030). Narrowed here, in
// TypeScript, `Extract` is exactly the right tool and the members keep the generator's own field
// names.
import {
  Action, Command, CoreConfig, CoreIn, CoreOut, DeviceClass, DriverEvent, Effect, Event, Form,
  Rating, TitleKey, WIRE_VERSION,
} from './contract';

/** The `http` member of `Effect`. */
export type EffectHttp = Extract<Effect, { kind: 'http' }>;

/** The `store` member of `Effect`. */
export type EffectStore = Extract<Effect, { kind: 'store' }>;

/** The `secret` member of `Effect`. */
export type EffectSecret = Extract<Effect, { kind: 'secret' }>;

/** The `player` member of `Effect`. */
export type EffectPlayer = Extract<Effect, { kind: 'player' }>;

/** The `view` member of `Effect`. */
export type EffectView = Extract<Effect, { kind: 'view' }>;

/**
 * Which member of `Effect` this is, as a plain string.
 *
 * A host switches on this and then asks for the member by name below. Written here because reading
 * a field off a union of anonymous shapes is a thing the `.ets` side should not have to do.
 */
export function effectKind(effect: Effect): string {
  return effect.kind;
}

export function asHttp(effect: Effect): EffectHttp {
  return effect as EffectHttp;
}

export function asStore(effect: Effect): EffectStore {
  return effect as EffectStore;
}

export function asSecret(effect: Effect): EffectSecret {
  return effect as EffectSecret;
}

export function asPlayer(effect: Effect): EffectPlayer {
  return effect as EffectPlayer;
}

export function asView(effect: Effect): EffectView {
  return effect as EffectView;
}

// --- the events a host sends back ---------------------------------------------------------------
//
// Each one is a member of the contract's `Event` union, which the generator writes as an anonymous
// shape. A function per member, so that no `.ets` file ever writes a contract literal: an object
// literal in ArkTS has to be typed by a named class or interface with no methods
// (`arkts-no-untyped-obj-literals`, 10605038), and an anonymous union member is neither.

export function networkEvent(online: boolean): Event {
  return { type: 'network', online };
}

export function driverEvent(generation: number, event: DriverEvent): Event {
  return { type: 'driver', event, generation };
}

export function addressEvent(address: string): Event {
  return { type: 'address', address };
}

export function actionEvent(action: Action, id?: string): Event {
  return id === undefined ? { type: 'action', action } : { type: 'action', id, action };
}

export function visibilityEvent(visible: boolean): Event {
  return { type: 'visibility', visible };
}

export function activityEvent(): Event {
  return { type: 'activity' };
}

export function unsupportedEvent(id: string): Event {
  return { type: 'unsupported', id };
}

export function storeDoneEvent(id: string, value?: string, keys?: string[]): Event {
  return { type: 'store_done', id, value, keys };
}

export function secretDoneEvent(id: string, value?: string): Event {
  return { type: 'secret_done', id, value };
}

export function httpDoneEvent(
  id: string,
  status: number,
  body?: string,
  headers?: Record<string, string>,
): Event {
  return { type: 'http_done', id, status, body, headers };
}

export function httpFailedEvent(id: string, reason: string): Event {
  return { type: 'http_failed', id, reason };
}

// --- the actions a screen raises -----------------------------------------------------------------

export function commandAction(command: Command): Action {
  return { action: 'command', command };
}

export function listCommand(title: TitleKey, inList: boolean): Command {
  return { command: 'list', in_list: inList, title };
}

export function rateCommand(title: TitleKey, rating?: Rating): Command {
  return { command: 'rate', rating, title };
}

export function hideCommand(title: TitleKey): Command {
  return { command: 'hide', title };
}

export function remindCommand(title: TitleKey, on: boolean): Command {
  return { command: 'remind', on, title };
}

export function enterProfileAction(profile: string, pin?: string): Action {
  return { action: 'enter_profile', profile, pin };
}

export function pauseAction(): Action {
  return { action: 'pause' };
}

export function stopAction(): Action {
  return { action: 'stop' };
}

export function requestAction(method: string, path: string, body?: string): Action {
  return { action: 'request', method, path, body };
}

// --- what crosses the ABI ------------------------------------------------------------------------
//
// Both of these are written and read here rather than in the host, for the same reason as
// everything else in this file: `CoreConfig.contract` is typed `WireDate`, a template literal type,
// and `CoreIn.events` is an array of anonymous union members. A host that built either literal in
// an `.ets` file would be writing a type ArkTS cannot name.

/** The config the core is opened with. `WIRE_VERSION` fills the contract field. */
export function coreConfigText(
  app: string,
  deviceClass: DeviceClass,
  form: Form,
  os: string,
  seedHex: string,
  version: string,
): string {
  const config: CoreConfig = {
    app,
    class: deviceClass,
    contract: WIRE_VERSION,
    form,
    mode: 'local',
    os,
    seed: seedHex,
    type: 'core_config',
    version,
  };
  return JSON.stringify(config);
}

/** One batch of events, as the core reads it. */
export function coreInText(events: Event[], monoMs: number, nowMs: number): string {
  const input: CoreIn = {
    contract: WIRE_VERSION,
    events,
    mono_ms: monoMs,
    now_ms: nowMs,
    type: 'core_in',
  };
  return JSON.stringify(input);
}

/** What the core answered. Null when the answer is not JSON, which a host treats as a refusal. */
export function readCoreOut(text: string): CoreOut | null {
  try {
    return JSON.parse(text) as CoreOut;
  } catch {
    return null;
  }
}
