// The words of the interface. A port of
// `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Theme.kt:62-134`.
//
// Every word a viewer reads comes from the server: 301 English keys in 18 languages, delivered at
// `/v1/strings/{language}` and held in `Seen.words` by the host. There is no English in this folder
// and there is no fallback string here either -- a missing key draws nothing, which is what the
// Kotlin does, because a key printed raw is worse than a blank and both are bugs in `strings/`.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { Seen } from '../../ports/Ports';

/** How the interface's language writes numbers, and its code for plural forms. `Theme.kt:74-86`. */
export class Speaking {
  /** The language's own code, for plural forms. */
  static code: string = 'en';
  /**
   * Its digits: `arab` for Arabic-Indic, `arabext` for Eastern Arabic-Indic / Persian, `latn` for
   * Western. The app sets this as the words arrive, from the `language.digits` key.
   */
  static digits: string = 'latn';

  /** Both, from the words a run has read. The root screen calls this once per change. */
  static read(seen: Seen): void {
    const digits = seen.words['language.digits'];
    Speaking.digits = digits === undefined || digits.length === 0 ? 'latn' : digits;
    Speaking.code = seen.language.length === 0 ? 'en' : seen.language;
  }
}

/** One word of the interface, or nothing. */
export function word(seen: Seen, key: string): string {
  const found = seen.words[key];
  return found === undefined ? '' : found;
}

/** A number as the interface's language writes it. `Theme.kt:89-97`. */
export function number(value: number | string): string {
  const plain = typeof value === 'string' ? value : value.toString();
  let base = 0;
  if (Speaking.digits === 'arab') {
    base = 0x0660;
  } else if (Speaking.digits === 'arabext') {
    base = 0x06F0;
  } else {
    return plain;
  }
  let made = '';
  for (let at = 0; at < plain.length; at++) {
    const code = plain.charCodeAt(at);
    made += code >= 0x30 && code <= 0x39 ? String.fromCharCode(base + code - 0x30) : plain.charAt(at);
  }
  return made;
}

/**
 * A word's `{name}` place filled with a value, isolated (U+2068, U+2069) so a number, a title or an
 * address keeps its own order inside a line of either direction. `Theme.kt:66-67`.
 *
 * `split` then `join` and not `replace`: the Kotlin's `String.replace(String, String)` replaces
 * every occurrence, and ArkTS's `replace(string, string)` replaces the first alone. A word with the
 * same place twice would otherwise come out half filled. A regular expression with the global flag
 * is the other way round it, and ArkTS refuses a regular expression LITERAL
 * (`arkts-no-regexp-literals`), so this avoids the engine altogether.
 */
export function fill(text: string, name: string, value: number | string): string {
  const shown = typeof value === 'string' ? value : number(value);
  return text.split('{' + name + '}').join('⁨' + shown + '⁩');
}

/** The plural form a count takes in a language, as CLDR names them. `Theme.kt:100-128`. */
export function pluralOf(n: number, lang: string): string {
  const base = lang.split('-')[0];
  if (base === 'ar') {
    if (n === 0) {
      return 'zero';
    }
    if (n === 1) {
      return 'one';
    }
    if (n === 2) {
      return 'two';
    }
    const hundred = n % 100;
    if (hundred >= 3 && hundred <= 10) {
      return 'few';
    }
    if (hundred >= 11 && hundred <= 99) {
      return 'many';
    }
    return 'other';
  }
  if (base === 'ru' || base === 'uk') {
    const ten = n % 10;
    const hundred = n % 100;
    if (ten === 1 && hundred !== 11) {
      return 'one';
    }
    if (ten >= 2 && ten <= 4 && !(hundred >= 12 && hundred <= 14)) {
      return 'few';
    }
    return 'many';
  }
  if (base === 'pl') {
    if (n === 1) {
      return 'one';
    }
    const ten = n % 10;
    const hundred = n % 100;
    if (ten >= 2 && ten <= 4 && !(hundred >= 12 && hundred <= 14)) {
      return 'few';
    }
    return 'many';
  }
  if (base === 'he' || base === 'iw') {
    if (n === 1) {
      return 'one';
    }
    if (n === 2) {
      return 'two';
    }
    return 'other';
  }
  if (base === 'ja' || base === 'ko' || base === 'zh') {
    return 'other';
  }
  if (base === 'fa') {
    return n === 1 ? 'one' : 'other';
  }
  if (base === 'fr' || base === 'pt' || base === 'hi') {
    return n <= 1 ? 'one' : 'other';
  }
  return n === 1 ? 'one' : 'other';
}

/**
 * A counted word: `{base}.{form}` for the count's plural form, else `other`, `many`, then `one`.
 * `Theme.kt:131-134`.
 */
export function counted(seen: Seen, base: string, n: number): string {
  const forms: string[] = [pluralOf(n, Speaking.code), 'other', 'many', 'one'];
  for (const form of forms) {
    const found = seen.words[base + '.' + form];
    if (found !== undefined) {
      return fill(found, 'n', n);
    }
  }
  return '';
}
