package streams.tv.browse;

import android.content.Context;
import android.graphics.Color;
import android.view.View;
import android.widget.TextView;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Round;
import streams.tv.views.Says;
import streams.tv.views.Shade;
import streams.tv.views.Type;

/**
 * The two cards a row is made of: the closed tile and the one the remote has opened.
 *
 * Every number in this file is a Netflix measurement or a derivation from one, and each one is
 * written as the integer literal it is written as at the line it came from. The provenance, carried
 * from {@code tv/browse/Rows.kt:99-102} rather than summarised:
 * {@code netflix-reference/tv/raw/tv-geometry-1920x1080.json}'s {@code row_screen.focused_tile} is
 * 906 x 510, matched to 904 x 508 in {@code 00-gaps.md} gap 9. {@code pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md:206} records
 * both, and the ten tests in {@code src/test/kotlin/streams/tv/NumbersTest.kt} hold the Kotlin
 * against that table; {@code src/test/java/streams/tv/browse/NumbersTest.java} holds this file
 * against the Kotlin, so the measurement has to survive the port to pass.
 */
final class Cards {
    private Cards() {
    }

    /**
     * A tile in a row, closed. {@code tv/browse/Rows.kt:59-66}.
     *
     * <h2>What a screen reader is handed, and why it is the title's name</h2>
     * The artwork <em>is</em> the title, so the title's name is what a person hears in place of
     * seeing it, and the badges drawn on top say the rest themselves ({@code Rows.kt:56-57}). The
     * name comes from the catalogue, which is already in the viewer's language, so no key and no
     * English is involved.
     */
    static final class Poster extends RefLayout {
        /** The closed tile, as the capture recorded it. {@code tv/browse/Rows.kt:61}. */
        static final int WIDE = 311;

        /** And its height, the same line. */
        static final int HIGH = 504;

        private final Art art;
        private View chips;

        Poster(Context context, Art.Source pictures) {
            super(context);
            // A tile clips its artwork to the brand's card corner, which is what `.clip(shapeOf(
            // Round.card))` does at `Rows.kt:61`. `RefLayout` leaves its children unclipped by
            // design, for the row that slides its posters in from past the edge of the screen, so
            // the clip is asked for here, by the part that wants it.
            setClipChildren(true);
            setClipToOutline(true);
            setBackground(Round.plate(context, Round.CARD, Ink.RAISED));
            art = new Art(context, pictures, null);
            addView(art, RefLayout.at(0, 0, WIDE, HIGH));
        }

        /** Points this tile at a title. Nothing is read twice, so a redraw is not a re-read. */
        void show(Feed.Card card) {
            art.show(card == null ? "" : card.tall, WIDE, HIGH);
            Says.onto(this, Says.says(card == null ? "" : card.name));
            if (chips != null) {
                removeView(chips);
                chips = null;
            }
            if (card != null && card.badges.length > 0) {
                chips = Parts.badges(getContext(), card.badges);
                // `Rows.kt:64`: bottom left of the tile, inset 12. Compose aligns it to the bottom
                // and pads; here the inset is a coordinate, and the y is the tile's height less the
                // inset, measured after the row has laid itself out.
                addView(chips, RefLayout.at(12, 0));
            }
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            super.onLayout(changed, left, top, right, bottom);
            if (chips != null) {
                // The badge row measures itself, so its top is only known here. `BottomStart` plus
                // `padding(12.ref)` at `Rows.kt:64` is the tile's foot less the badge's own height
                // and less the same inset.
                int inset = Ref.px(this, 12);
                int y = getHeight() - inset - chips.getMeasuredHeight();
                int x = isMirrored() ? getWidth() - inset - chips.getMeasuredWidth() : inset;
                chips.layout(x, y, x + chips.getMeasuredWidth(), y + chips.getMeasuredHeight());
            }
        }
    }

    /**
     * The card the remote has opened: it stays still and its art crossfades.
     * {@code tv/browse/Rows.kt:80-112}.
     *
     * <h2>Why the name and the chips are drawn over the art</h2>
     * Netflix composites them into the artwork instead, which
     * {@code notes/netflix-reference/00-gaps.md} records as gap 4 and as an open owner decision, so
     * this stays as it is until he takes it. {@code Rows.kt:72-78} carries the rest of that history:
     * {@code 505652c} swapped the name for the title's LOGO asset in a 240 x 83 box inset 17 and 23
     * and moved the chips to the far corner at 20 and 30, and none of those five numbers is measured
     * anywhere in {@code notes/netflix-reference}, so they were removed rather than kept on an
     * unfiled reading of a frame. They are not re-introduced here.
     *
     * <h2>The card is always the one the remote is on</h2>
     * So its reading is {@code saysItsWords(on = true)} ({@code Rows.kt:91}): the name it draws is
     * its label, and it says the remote is on it. It is the one card of the row that is open, so
     * there is no state in which that is false.
     */
    static final class Open extends RefLayout {
        /** The focused card, matched to the capture. {@code tv/browse/Rows.kt:88}. */
        static final int WIDE = 904;

        /** And its height, the same line. */
        static final int HIGH = 508;

        /** The inset the name and the chips share. {@code tv/browse/Rows.kt:97} and {@code :103}. */
        static final int INSET = 24;

        /**
         * The name's own width: the card less that inset on each side. A derivation, and the one
         * {@code NumbersTest.kt}'s seventh test was written for -- a card widened with the name's
         * width left behind reads as a drawing bug and is a transcription one. 856 is the number at
         * {@code Rows.kt:103} and {@code WIDE - 2 * INSET} is where it comes from; the test holds the
         * two together on both sides of the port.
         */
        static final int NAME_WIDE = WIDE - 2 * INSET;

        private final Art art;
        private final Art fading;
        private final View foot;
        private final TextView name;
        private final View edge;
        private View chips;
        private String showing = "";

        Open(Context context, Art.Source pictures) {
            super(context);
            setClipChildren(true);
            setClipToOutline(true);
            setBackground(Round.plate(context, Round.CARD, Ink.RAISED));

            // Two pictures and not one, because `Crossfade` at `Rows.kt:93` fades the art of the
            // title that has gone out to the art of the one that has come in, over SLIDE_MS. One
            // ImageView cannot hold both frames, so the one going out is kept underneath and faded.
            fading = new Art(context, pictures, null);
            addView(fading, RefLayout.at(0, 0, WIDE, HIGH));
            art = new Art(context, pictures, null);
            addView(art, RefLayout.at(0, 0, WIDE, HIGH));

            // `Rows.kt:96`: the foot shade, the card's full width, 200 high, at its bottom.
            foot = new View(context);
            // `Rows.kt:53`: transparent at the card's top edge of this box, black at 0.7 strength
            // at its bottom. Two stops at 0 and 1, which is the one gradient on this page that is
            // evenly spaced.
            foot.setBackground(Shade.down(new int[] {Color.TRANSPARENT, Ink.black(70)},
                    new float[] {0f, 1f}));
            Says.onto(foot, Says.decoration());
            addView(foot, RefLayout.at(0, HIGH - 200, WIDE, 200));

            name = Type.line(context, 44, Type.HEAVY, Color.WHITE);
            addView(name, RefLayout.at(INSET, 0, NAME_WIDE, RefLayout.WRAP));

            // `Rows.kt:110`: the white focus border, drawn over everything else, on the card's own
            // corner. A border and not a ring the platform draws: `RemoteHost` turns the platform's
            // own focus highlight off precisely so a viewer does not see two.
            edge = new View(context);
            edge.setBackground(Round.edged(context, Round.CARD, Color.TRANSPARENT, 3, Color.WHITE));
            Says.onto(edge, Says.decoration());
            addView(edge, RefLayout.at(0, 0, WIDE, HIGH));

            Says.onto(this, Says.saysItsWords(true));
        }

        /**
         * Points the card at a title.
         *
         * @param fade true to crossfade from what it was showing, which is every move along a row,
         *             and false to arrive already showing it, which is a page the viewer has just
         *             come back to
         */
        void show(Feed.Card card, boolean fade, int slideMs) {
            String wanted = card == null ? "" : card.wide;
            name.setText(card == null ? "" : card.name);
            if (chips != null) {
                removeView(chips);
                chips = null;
            }
            if (card != null && card.badges.length > 0) {
                chips = Parts.badges(getContext(), card.badges);
                addView(chips, RefLayout.at(INSET, 0));
            }
            if (wanted.equals(showing)) {
                return;
            }
            if (fade && !showing.isEmpty()) {
                fading.setImageDrawable(art.getDrawable());
                fading.setAlpha(1f);
                art.setAlpha(0f);
                art.show(wanted, WIDE, HIGH);
                art.animate().cancel();
                art.animate().alpha(1f).setDuration(slideMs).start();
                fading.animate().cancel();
                fading.animate().alpha(0f).setDuration(slideMs).start();
            } else {
                art.animate().cancel();
                art.setAlpha(1f);
                fading.setAlpha(0f);
                art.show(wanted, WIDE, HIGH);
            }
            showing = wanted;
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            super.onLayout(changed, left, top, right, bottom);
            // The name and the chips sit on the card's foot, so their tops depend on how tall they
            // measured. `Rows.kt:103` puts the name 16 off the bottom and `:97` puts the chips 78
            // off it, both inset 24 from the start.
            int inset = Ref.px(this, INSET);
            place(name, inset, Ref.px(this, 16));
            if (chips != null) {
                place(chips, inset, Ref.px(this, 78));
            }
        }

        private void place(View child, int inset, int up) {
            int y = getHeight() - up - child.getMeasuredHeight();
            int x = isMirrored() ? getWidth() - inset - child.getMeasuredWidth() : inset;
            child.layout(x, y, x + child.getMeasuredWidth(), y + child.getMeasuredHeight());
        }
    }
}
