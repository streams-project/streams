package streams.tv.browse;

import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.Drawable;
import android.view.Gravity;
import android.view.View;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.Round;
import streams.tv.views.Says;
import streams.tv.views.Type;
import streams.tv.views.WordText;
import streams.tv.views.Words;

/**
 * The small parts the hero and the rows are built out of: a badge row, a line of facts, a large
 * button, a profile's face.
 *
 * <h2>This duplicates {@code tv/parts/} and that is deliberate and temporary</h2>
 * {@code parts/Badge.kt}, {@code parts/Facts.kt}, {@code parts/Pill.kt} and {@code parts/Face.kt}
 * are composables, and a composable cannot be a child of a {@code ViewGroup}. Note 210 gives
 * {@code tv/.../parts/**} to wave 2's package T4 and {@code tv/.../browse/**} to T1, so the parts
 * this page draws have to exist in Views before the package that owns them is converted, or the
 * hero has no buttons and the key map's HERO row cannot be pressed at all.
 *
 * So these four are written here, against the numbers in those four files, and when T4 lands they
 * move into a Views {@code parts} package and this file holds nothing. What makes the duplication
 * safe for as long as it lasts is that the numbers are held equal by a test rather than by care:
 * {@code src/test/java/streams/tv/browse/NumbersTest.java} reads each number out of the Kotlin and
 * out of this file and asserts them equal, so a badge resized in one place and not the other fails.
 *
 * Everything here is one level of nesting at most, which is note 214's constraint: a badge row and a
 * facts line are a handful of children that have to sit beside each other and measure themselves, so
 * they are the one case on this page where a {@link LinearLayout} is the honest answer and absolute
 * coordinates are not -- a fact's width is the width of a word nobody measured.
 */
final class Parts {
    private Parts() {
    }

    /**
     * The badges on a tile, as small pills. {@code parts/Badge.kt:32-60}.
     *
     * No reading is put on this row or on the labels inside it, which matches what the Compose path
     * does today: {@code Poster} applies {@code says(tile.name)} without gathering
     * ({@code tv/app/Spoken.kt:41-42} sets a description and nothing else), so a badge's own words
     * stay a node of their own in both toolkits. A badge carries drawn text, so it is a node with
     * text and not a silent stop, which is what note 210's wave 2 check reads for.
     */
    static LinearLayout badges(Context context, String[] said) {
        LinearLayout row = new LinearLayout(context);
        row.setOrientation(LinearLayout.HORIZONTAL);
        row.setGravity(Gravity.CENTER_VERTICAL);
        for (int at = 0; at < said.length; at++) {
            if (said[at] == null || said[at].isEmpty()) {
                continue;
            }
            TextView chip = Type.line(context, 22, Type.BOLD, Color.WHITE);
            chip.setText(said[at]);
            chip.setBackground(Round.plate(context, Round.CHIP, Ink.black(72)));
            chip.setPadding(Ref.px(context, 12), Ref.px(context, 4), Ref.px(context, 12),
                    Ref.px(context, 4));
            chip.setGravity(Gravity.CENTER);
            LinearLayout.LayoutParams spot = new LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT);
            // `Arrangement.spacedBy(8.ref)` at `parts/Badge.kt:40`: the gap is between badges and
            // not before the first, so it goes on the start of every badge but the first.
            if (row.getChildCount() > 0) {
                spot.leftMargin = Ref.px(context, 8);
            }
            row.addView(chip, spot);
        }
        return row;
    }

    /**
     * The line of facts about a title, in one order on every screen: genre, year, length, age.
     * {@code parts/Facts.kt:52-62}. The words and the order are the seam's, because the counted
     * forms and the digits belong to {@code shared/Theme.kt} and are not restated here.
     */
    static LinearLayout facts(Context context, String[] said) {
        LinearLayout row = new LinearLayout(context);
        row.setOrientation(LinearLayout.HORIZONTAL);
        row.setGravity(Gravity.CENTER_VERTICAL);
        for (int at = 0; at < said.length; at++) {
            if (said[at] == null || said[at].isEmpty()) {
                continue;
            }
            if (row.getChildCount() > 0) {
                View dot = new View(context);
                dot.setBackground(Round.round(Ink.MUTED));
                Says.onto(dot, Says.decoration());
                LinearLayout.LayoutParams round = new LinearLayout.LayoutParams(
                        Ref.px(context, 6), Ref.px(context, 6));
                round.leftMargin = Ref.px(context, 14);
                round.rightMargin = Ref.px(context, 14);
                row.addView(dot, round);
            }
            TextView fact = Type.line(context, 24, Type.BOLD, Color.WHITE);
            fact.setText(said[at]);
            row.addView(fact);
        }
        return row;
    }

    /**
     * A large button. In focus it turns white. {@code parts/Pill.kt:38-52}.
     *
     * Its word is its label for a screen reader and the icon beside it is decoration, because the
     * icon repeats the word ({@code Pill.kt:33-34}). A pill with no word and no icon draws nothing
     * rather than an empty white shape -- {@code Pill.kt:35-36} records that dead end as measured on
     * the Streamer on 2026-11-11, a {@code Pill("")} on black with every word around it missing --
     * so this returns a view whose visibility follows its words, set in {@link Pill#words}.
     */
    static final class Pill extends LinearLayout implements Words.Bound {
        private final WordText word;
        private final ImageView mark;
        private boolean on;

        Pill(Context context, String key, Drawable icon) {
            super(context);
            setOrientation(HORIZONTAL);
            setGravity(Gravity.CENTER_VERTICAL);
            setPadding(Ref.px(context, 46), 0, Ref.px(context, 46), 0);
            if (icon != null) {
                mark = new ImageView(context);
                mark.setImageDrawable(icon);
                Says.onto(mark, Says.decoration());
                LayoutParams size = new LayoutParams(Ref.px(context, 34), Ref.px(context, 34));
                size.rightMargin = Ref.px(context, 12);
                addView(mark, size);
            } else {
                mark = null;
            }
            word = WordText.of(context, key, 32);
            word.setTypeface(Type.face(context, Type.BOLD));
            word.setMaxLines(1);
            addView(word);
            lit(false);
        }

        /** The two readings of {@code Pill.kt:41-42}, which is the whole of its focus. */
        void lit(boolean on) {
            this.on = on;
            setBackground(Round.pill(getContext(), 84, on ? Color.WHITE : Ink.white(16)));
            int ink = on ? Ink.BACK : Color.WHITE;
            word.setTextColor(ink);
            if (mark != null) {
                mark.setColorFilter(ink);
            }
            Says.onto(this, Says.saysItsWords(on));
        }

        boolean lit() {
            return on;
        }

        @Override
        public void words(Words words) {
            word.words(words);
            // `Pill.kt:40`: no word and no icon draws nothing at all. The icon is the one thing that
            // can still carry the button on its own, which is what `R.drawable.icon_play` does on
            // the hero's first button.
            boolean has = word.getText().length() > 0 || mark != null;
            setVisibility(has ? VISIBLE : GONE);
        }
    }

    /**
     * A profile's picture, or its first letter when it has none. {@code shared/Face.kt:26-53} and
     * {@code parts/Face.kt:16-25}.
     *
     * The letter's size is a share of the face's, {@code size * 46 / 100} at {@code parts/Face.kt:23},
     * so one number makes every face on the surface. The plate behind it is {@code Accent}, or
     * {@code Join} for a child's profile.
     *
     * It says nothing of its own here, which is {@code shared/Face.kt:38}'s default and the right
     * answer at this call site: the bar's face sits inside a part that already says whose it is
     * ({@code tv/browse/TopBar.kt:83}), so a reading here would say the name twice. A face standing
     * alone passes its words to {@link Says#says(String)} at its own call site.
     */
    static final class Face extends android.widget.FrameLayout {
        private final TextView letter;
        private final Art picture;

        Face(Context context, Art.Source pictures, int refSize) {
            super(context);
            setClipToOutline(true);
            letter = Type.line(context, refSize * 46 / 100, Type.HEAVY, Color.WHITE);
            letter.setGravity(Gravity.CENTER);
            addView(letter, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT));
            picture = new Art(context, pictures, null);
            addView(picture, new LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.MATCH_PARENT));
            Says.onto(this, Says.decoration());
        }

        /**
         * @param corner the face's corner, which is a share of its size ({@code Round.kt:46}), so it
         *               is given rather than taken from a constant
         */
        void show(Feed.Who who, int corner, boolean ringed) {
            boolean kids = who != null && who.kids;
            int plate = kids ? Ink.JOIN : Ink.ACCENT;
            setBackground(ringed
                    ? Round.edged(getContext(), corner, plate, 3, Color.WHITE)
                    : Round.plate(getContext(), corner, plate));
            String avatar = who == null ? "" : who.picture;
            picture.show(avatar, 0, 0);
            picture.setVisibility(avatar.isEmpty() ? GONE : VISIBLE);
            String name = who == null ? "" : who.name;
            // `Locale.ROOT` and not the default locale, which is what Kotlin's `uppercase()` at
            // `shared/Face.kt:44` does. The default locale is the device's, so a Turkish television
            // would draw a dotless capital for a name beginning with i and a name beginning with I
            // would not match the one drawn on a phone.
            letter.setText(name.isEmpty() ? "" : name.substring(0, 1).toUpperCase(java.util.Locale.ROOT));
        }
    }
}
