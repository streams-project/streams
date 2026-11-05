// Audio slot. No sound ships yet: with no source in the config this does nothing.

export interface AudioSlot {
  sync(ms: number): void;
}

export function audioSlot(config: { src: string | null; startAtMs: number; volume: number }): AudioSlot {
  if (!config || !config.src) return { sync: function () {} };
  var element = new Audio(config.src);
  element.preload = "auto";
  element.volume = config.volume;
  var started = false;
  return {
    sync: function (ms: number): void {
      if (started || ms < config.startAtMs) return;
      started = true;
      var played: unknown = element.play();
      // Older sets return nothing from play().
      if (played && (played as { catch?: unknown }).catch) (played as { catch(f: () => void): void }).catch(function () {});
    }
  };
}
