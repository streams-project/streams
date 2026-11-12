// Which picture goes in which place. A port of
// `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Art.kt:14-22`, so a Roku chooses the
// same artwork as every other surface.

import type { Image } from "./contract";

function find(images: Image[] | undefined, role: string): string | null {
  return images?.find((image) => image.role === role)?.url ?? null;
}

/** The picture for a wide place. (`Art.kt:17-19`) */
export function wide(images: Image[] | undefined): string | null {
  return find(images, "backdrop") ?? find(images, "still") ?? find(images, "poster");
}

/** The picture for a tall place. (`Art.kt:21-22`) */
export function tall(images: Image[] | undefined): string | null {
  return find(images, "poster") ?? wide(images);
}

/** A title's own lettering, when it has some. (`TitlePage.kt:195`) */
export function logo(images: Image[] | undefined): string | null {
  return find(images, "logo");
}
