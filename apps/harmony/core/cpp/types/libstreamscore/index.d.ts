// What napi_bridge.c exports, as ArkTS reads it. Four functions and no rules.
// `step` throws when the core refuses a message; the core's own statuses are in
// platform/bridges/c/core.h.
export const abi: () => number;
/** A handle, or 0 when the config did not read. */
export const open: (config: string) => number;
export const step: (handle: number, input: string) => string;
export const close: (handle: number) => void;
