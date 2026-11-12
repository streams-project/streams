// A profile's picture, or its first letter. A port of
// `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Face.kt`.
//
// [says] is the words a screen reader is given. The default is none, "because the letter standing
// in for a missing picture is not a word -- a reader would say J and stop -- and because a face is
// almost always drawn inside a control that already says whose it is. A face that stands alone
// passes the person's name here" (`Face.kt:20-25`).
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.
import { Profile } from '../../contract/contract';
import { Pictures } from '../../ports/Ports';
import { Art } from './Pictures';
import { SKIP_ALL } from './Heard';
import { ACCENT, INK, JOIN } from './Theme';

@ComponentV2
export struct Face {
  @Param person: Profile | null = null;
  @Require @Param server: string;
  @Require @Param pictures: Pictures;
  /** The box, in vp. Square, as every call site's is. */
  @Require @Param box: number;
  @Require @Param corner: number;
  /** The first letter's size when there is no picture. */
  @Param letterType: number = 0;
  @Param says: string = '';

  build() {
    Stack() {
      if (this.avatar() !== null) {
        Art({
          url: this.avatar(),
          wide: this.box,
          high: this.box,
          pictures: this.pictures,
        })
      } else {
        Text(this.letter())
          .fontSize(this.letterType)
          .fontWeight(FontWeight.Bolder)
          .fontColor(INK)
          .maxLines(1)
      }
    }
    .width(this.box)
    .height(this.box)
    // A kid's profile wears the brand's join colour, everyone else's the accent. `Face.kt:36`.
    .backgroundColor(this.person !== null && this.person.kids ? JOIN : ACCENT)
    .borderRadius(this.corner)
    .clip(true)
    .accessibilityLevel(this.says.length === 0 ? SKIP_ALL : 'auto')
    .accessibilityText(this.says)
  }

  /** `Face.kt:35`: an avatar is only a picture when it is an address. */
  private avatar(): string | null {
    const person = this.person;
    if (person === null) {
      return null;
    }
    const avatar = person.avatar;
    if (avatar === undefined) {
      return null;
    }
    if (avatar.startsWith('/')) {
      return this.server + avatar;
    }
    return avatar.startsWith('http') ? avatar : null;
  }

  private letter(): string {
    const person = this.person;
    if (person === null || person.name.length === 0) {
      return '';
    }
    return person.name.substring(0, 1).toUpperCase();
  }
}
