// The build's own settings, and the app's manifest.
//
// The RSG 2.0 SDK reads `roku-config.ts` from the project root and can generate the whole manifest
// from `app.manifestEntries`, with no `pkg/manifest` file on disk at all (verified 2026-11-12,
// https://developer.roku.com/dev/docs/rsg-sdk-standing-docs-core-concepts-manifest-manifest-processing:
// "No `pkg/manifest` exists. The build generates the manifest from `app.manifestEntries`"). That is
// what this file does, for one reason: the app's name is the brand's, and the brand's name lives in
// `brand.json` and nowhere else. A manifest file on disk would be a second place holding it.
//
// Every key below appears in that page's own examples. `rsg_version` is the one key the compiler
// checks: "2.0" is current, and an absent key means "1.2".

import brand from "../../brand.json";

/** The icons and splash screens a Roku package carries. Written by the brand step, not by hand. */
const ART = {
  mm_icon_focus_fhd: "pkg:/images/icon_540x405.png",
  mm_icon_focus_hd: "pkg:/images/icon_336x210.png",
  splash_screen_fhd: "pkg:/images/splash_1920x1080.jpg",
  splash_screen_hd: "pkg:/images/splash_1280x720.jpg",
};

export default {
  app: {
    manifestEntries: {
      /** The brand's name, from `brand.json`. No label in this folder is written in English. */
      title: brand.name,
      major_version: 1,
      minor_version: 0,
      build_version: 1,
      ...ART,
      /**
       * One resolution, 1920 by 1080. The television surface measures in reference pixels on that
       * same grid (`apps/android/surfaces/shared/.../Measure.kt:9-12`), so every number in
       * `src/numbers.ts` crosses with no scaling.
       */
      ui_resolutions: "fhd",
      rsg_version: "2.0",
    },
  },
};
