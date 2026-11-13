package streams.tv.parts;

import android.content.Context;
import android.graphics.Color;
import android.view.Gravity;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.TextView;
import java.util.Locale;
import streams.shared.Ink;
import streams.shared.Says;
import streams.shared.Type;
import streams.tv.views.Round;

/**
 * A profile's picture, or its first letter when it has none. Every screen draws this one part --
 * eight files and thirteen call sites on 2026-11-13. The Views twin of {@code parts/Face.kt:16-25}
 * over {@code shared/Face.kt:26-53}.
 *
 * <h2>Why it holds a slot and not a picture port</h2>
 * {@code browse/Art.java} reads pictures through the {@code Pictures} port, and this package may
 * not call it: {@code browse} draws with {@code parts} -- {@code Pill}, {@code Facts},
 * {@code Badges} and this -- so an import the other way would make the two folders a cycle, and the
 * next folder to convert would inherit it. Declaring a second picture interface here instead would
 * give the port's shape a second home, which is the thing note 210 section 4 is trying to end.
 *
 * So the face draws the plate, the letter and the ring, and {@link #setPicture(View)} takes the
 * view that paints the avatar. The caller already owns one: {@code browse} passes its {@code Art},
 * and the screens still to convert pass whatever their own wave gives them. When wave 0's package
 * S2 turns {@code Pictures} into a Java callable port, the slot takes it with no change here.
 *
 * <h2>What it says</h2>
 * Nothing, by default, and that is {@code shared/Face.kt:38}'s own default. A face almost always
 * sits inside a part that already says whose it is, and a reading here would say the name twice.
 * A face standing on its own is given {@link #says(String)} -- the person's name, which is already
 * in the viewer's language because they typed it ({@code parts/Face.kt:8-12}).
 */
public final class Face extends FrameLayout {
    /** The letter's size as a share of the face's, in hundredths. {@code parts/Face.kt:23}. */
    static final int LETTER_SHARE = 46;

    static final int LETTER_SHARE_OF = 100;

    private final TextView letter;
    private View picture;
    private int corner = Round.CARD * Round.FACE_SHARE / Round.FACE_SHARE_OF;
    private boolean kids;
    private boolean ringed;

    /**
     * @param refSize the face's size as it was measured, which is the one number that makes the
     *                letter: {@code size * 46 / 100} at {@code parts/Face.kt:23}, so a face drawn
     *                at any size carries the same letter in proportion
     */
    public Face(Context context, int refSize) {
        super(context);
        setClipToOutline(true);
        letter = Type.line(context, refSize * LETTER_SHARE / LETTER_SHARE_OF, Type.HEAVY, Color.WHITE);
        letter.setGravity(Gravity.CENTER);
        letter.setEllipsize(null);
        addView(letter, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT));
        Says.onto(this, Says.decoration());
        plate();
    }

    /**
     * The view that paints the avatar, filling the face. It is hidden whenever there is no picture,
     * so the letter behind it shows through -- {@code shared/Face.kt:41-52} draws one or the other
     * and never both.
     */
    public void setPicture(View picture) {
        if (this.picture != null) {
            removeView(this.picture);
        }
        this.picture = picture;
        if (picture != null) {
            addView(picture, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT));
            picture.setVisibility(GONE);
        }
    }

    /**
     * The face's corner. {@code app/Round.kt:46} makes it a share of the card's corner,
     * {@code card * 100 / 130}, which is the default here. The add tile and the focus ring draw the
     * same shape at a different size and set their own.
     */
    public void setCorner(int corner) {
        this.corner = corner;
        plate();
    }

    /** The white ring a face in focus wears. {@code tv/profiles/ProfileCard.kt}. */
    public void setRinged(boolean ringed) {
        this.ringed = ringed;
        plate();
    }

    /**
     * Who this is.
     *
     * @param name    the person's name, whose first letter is drawn when there is no picture
     * @param hasKids true for a child's profile, which changes the plate behind the letter
     */
    public void show(String name, boolean hasKids) {
        this.kids = hasKids;
        plate();
        String said = name == null ? "" : name;
        // `Locale.ROOT` and not the device's locale, which is what Kotlin's `uppercase()` at
        // `shared/Face.kt:44` does. A Turkish television would otherwise draw a dotless capital for
        // a name beginning with i, and the same name would not match the letter drawn on a phone.
        letter.setText(said.isEmpty() ? "" : said.substring(0, 1).toUpperCase(Locale.ROOT));
    }

    /**
     * The avatar, already made absolute against the server, or empty for the letter.
     *
     * The rule for what counts as one is {@code shared/Face.kt:36}: an address that starts with
     * {@code /} or {@code http}. Anything else is not a picture and the letter is drawn, which is
     * what an empty avatar field and a stray word both arrive as.
     */
    public void showPicture(boolean painted) {
        if (picture != null) {
            picture.setVisibility(painted ? VISIBLE : GONE);
        }
    }

    /** The words a reader is given, for a face that stands on its own with no name beside it. */
    public void says(String said) {
        Says.onto(this, said == null || said.isEmpty() ? Says.decoration() : Says.says(said));
    }

    private void plate() {
        int colour = kids ? Ink.JOIN : Ink.ACCENT;
        setBackground(ringed
                ? Round.edged(getContext(), corner, colour, 3, Color.WHITE)
                : Round.plate(getContext(), corner, colour));
    }
}
