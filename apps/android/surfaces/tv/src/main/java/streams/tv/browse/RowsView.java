package streams.tv.browse;

import android.animation.ValueAnimator;
import android.content.Context;
import android.graphics.Color;
import android.view.View;
import android.view.animation.PathInterpolator;
import android.widget.TextView;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Round;
import streams.tv.views.Says;
import streams.tv.views.Type;

/**
 * The two rows a browse page draws: the one that owns the screen, and the dim one below it.
 *
 * {@code tv/browse/Rows.kt:114-178}. Every coordinate here is the coordinate on that file's line,
 * and the five constants are its {@code :44-48}.
 *
 * <h2>Why there is no {@code RecyclerView} and no adapter</h2>
 * Verified in wave 1 and measured in note 210's decision 1: this surface has zero lazy lists
 * ({@code rg -o "LazyColumn\(|LazyRow\(|LazyVerticalGrid\("} over {@code tv/src/main/kotlin} returns
 * 0 matches; the phone returns 12), and the widest row is one open card plus {@code AHEAD = 5}
 * posters ahead, one behind, and {@code NEXT_POSTERS = 6} on the dim row below. At most a dozen
 * children, every one of them on screen. So the children are made once and pointed at different
 * titles, which is what a recycler does and what a fixed set of twelve views does not need a
 * recycler to do. Note 213's open gate -- D-pad focus traversal across a recycling boundary -- does
 * not arise, because there is no boundary.
 */
final class RowsView {
    /** The tile pitch: the tile plus the gutter beside it. {@code tv/browse/Rows.kt:44}. */
    static final int STEP = 328;

    /** How many posters past the open card a row holds. {@code tv/browse/Rows.kt:45}. */
    static final int AHEAD = 5;

    /** And how many the dim row below shows. {@code tv/browse/Rows.kt:46}. */
    static final int NEXT_POSTERS = 6;

    /** How long a move along a row takes. {@code tv/browse/Rows.kt:47}. */
    static final int SLIDE_MS = 240;

    /** And how long the words under it take to arrive. {@code tv/browse/Rows.kt:48}. */
    static final int FADE_MS = 200;

    /**
     * The curve every move on this page uses. {@code tv/browse/Rows.kt:51}'s
     * {@code CubicBezierEasing(0.22f, 0.61f, 0.2f, 1f)}.
     *
     * {@code PathInterpolator} takes the same four control points and is the exact twin, which is
     * note 210 section 6's own row for this number. Both are a cubic Bezier through (0,0) and (1,1)
     * with those two handles, so nothing about the motion changes in the port.
     */
    static PathInterpolator glide() {
        return new PathInterpolator(0.22f, 0.61f, 0.2f, 1f);
    }

    private RowsView() {
    }

    /**
     * The row that owns the screen: its focused title opens wide with the next ones beside it.
     * {@code tv/browse/Rows.kt:115-156}.
     */
    static final class Open extends RefLayout {
        /** Where the first poster beside the open card starts. {@code tv/browse/Rows.kt:137}. */
        static final int AHEAD_X = Round.PAGE_EDGE + Cards.Open.WIDE + 19;

        /** Where the poster the viewer came from peeks in. {@code tv/browse/Rows.kt:132}. */
        static final int BEHIND_X = -264;

        /** The band the posters of this row sit in. {@code tv/browse/Rows.kt:132} and {@code :137}. */
        static final int POSTER_Y = 202;

        /** And the open card's own y. {@code tv/browse/Rows.kt:87}. */
        static final int CARD_Y = 200;

        private final TextView title;
        private final Cards.Poster behind;
        private final Cards.Poster[] ahead = new Cards.Poster[AHEAD];
        private final Cards.Open card;
        private final TextView story;
        private View facts;

        /** Where the row's posters are now, in tiles. The thing {@code animateFloatAsState} held. */
        private float moved;
        private ValueAnimator sliding;
        private Feed.Row row;
        private int at;

        Open(Context context, Art.Source pictures) {
            super(context);
            title = Type.line(context, 32, Type.BOLD, Color.WHITE);
            addView(title, RefLayout.at(Round.PAGE_EDGE, 146));

            behind = new Cards.Poster(context, pictures);
            // `.alpha(0.55f)` at `Rows.kt:132`: the tile the viewer came from is dim.
            behind.setAlpha(0.55f);
            addView(behind, RefLayout.at(BEHIND_X, POSTER_Y, Cards.Poster.WIDE, Cards.Poster.HIGH));

            for (int slot = 0; slot < AHEAD; slot++) {
                ahead[slot] = new Cards.Poster(context, pictures);
                addView(ahead[slot],
                        RefLayout.at(AHEAD_X, POSTER_Y, Cards.Poster.WIDE, Cards.Poster.HIGH));
            }

            card = new Cards.Open(context, pictures);
            addView(card, RefLayout.at(Round.PAGE_EDGE, CARD_Y, Cards.Open.WIDE, Cards.Open.HIGH));

            // `Rows.kt:143` and `:146`: the facts at 728 and the story at 768, both on the page
            // margin, both faded in together when the detail arrives (`:142`). They are two
            // children at two measured coordinates rather than one box holding both, so each
            // number stays the literal it is on the line it came from -- a wrapper at 728 would
            // turn 768 into an offset of 40 and the second measurement would be gone from the
            // source. The two alphas are only ever written by `tell`, which is the single writer
            // rule note 210 names as the mitigation for a missed redraw.
            story = Type.story(context, 30, 39, 3, Ink.white(80));
            story.setAlpha(0f);
            addView(story, RefLayout.at(Round.PAGE_EDGE, 768, 1000, RefLayout.WRAP));
        }

        /**
         * Draws a row with the remote on one of its tiles.
         *
         * @param slide true to animate the posters to their new places, which is a move along the
         *              row, and false to arrive with them already there, which is a row the page has
         *              just risen to
         */
        void show(Feed.Row row, int at, boolean slide) {
            this.row = row;
            this.at = at;
            title.setText(row == null ? "" : row.title);
            Says.onto(title, Says.means(row == null ? "" : row.title));
            card.show(row == null ? null : row.at(at), slide, SLIDE_MS);

            boolean hasBehind = at > 0 && row != null;
            behind.setVisibility(hasBehind ? VISIBLE : GONE);
            if (hasBehind) {
                behind.show(row.at(at - 1));
            }

            int last = row == null ? -1 : row.cards.size() - 1;
            for (int slot = 0; slot < AHEAD; slot++) {
                int index = at + 1 + slot;
                boolean has = row != null && index <= Math.min(at + AHEAD, last);
                ahead[slot].setVisibility(has ? VISIBLE : GONE);
                if (has) {
                    ahead[slot].show(row.at(index));
                }
            }

            if (slide) {
                slideTo(at);
            } else {
                stopSliding();
                moved = at;
                place();
            }
        }

        /** What is known about the open title, faded in as {@code Rows.kt:124} and {@code :142} do. */
        void tell(Feed.Story told) {
            if (facts != null) {
                removeView(facts);
                facts = null;
            }
            if (told == null) {
                fade(0f);
                story.setText("");
                return;
            }
            facts = Parts.facts(getContext(), told.facts);
            facts.setAlpha(0f);
            addView(facts, RefLayout.at(Round.PAGE_EDGE, 728));
            story.setText(told.overview);
            fade(1f);
        }

        /** Both halves of what is told, moved together, which is {@code Rows.kt:142}'s one alpha. */
        private void fade(float to) {
            story.animate().cancel();
            story.animate().alpha(to).setDuration(FADE_MS).start();
            if (facts != null) {
                facts.animate().cancel();
                facts.animate().alpha(to).setDuration(FADE_MS).start();
            }
        }

        /**
         * Moves the posters to where the row stands now.
         *
         * {@code animateFloatAsState(at.toFloat(), tween(SLIDE_MS, easing = Glide))} at
         * {@code Rows.kt:122} is a value that chases a target and restarts from wherever it has got
         * to when the target changes mid flight. That is what this is: the animator is cancelled at
         * its current {@code moved} and a fresh one runs from there, which is note 210's "manual
         * cancellation when a key arrives mid flight" and is what keeps a held arrow smooth.
         */
        private void slideTo(final int target) {
            if (moved == target) {
                place();
                return;
            }
            stopSliding();
            sliding = ValueAnimator.ofFloat(moved, target);
            sliding.setDuration(SLIDE_MS);
            sliding.setInterpolator(glide());
            sliding.addUpdateListener(new ValueAnimator.AnimatorUpdateListener() {
                @Override
                public void onAnimationUpdate(ValueAnimator running) {
                    moved = ((Float) running.getAnimatedValue()).floatValue();
                    place();
                }
            });
            sliding.start();
        }

        private void stopSliding() {
            if (sliding != null) {
                sliding.cancel();
                sliding = null;
            }
        }

        /**
         * Where each poster beside the card is, this frame.
         *
         * {@code Rows.kt:137}'s {@code translationX = (index - 1 - moved) * step}, with
         * {@code step = STEP.ref.toPx() * along()} from {@code :121}. {@code along()} is 1 left to
         * right and -1 right to left ({@code app/Round.kt:59}), because in a right to left language
         * the row runs leftwards and every move along it turns. Here the sign comes from
         * {@link RefLayout#isMirrored()}, which is the one switch the whole layout mirrors on, so
         * there is no second reading of a layout direction.
         */
        private void place() {
            float step = Ref.px(this, STEP) * (isMirrored() ? -1 : 1);
            for (int slot = 0; slot < AHEAD; slot++) {
                int index = at + 1 + slot;
                ahead[slot].setTranslationX((index - 1 - moved) * step);
            }
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            super.onLayout(changed, left, top, right, bottom);
            place();
        }
    }

    /**
     * The next row, dim, cut by the screen's edge. {@code tv/browse/Rows.kt:162-178}.
     *
     * It shows where the viewer left it: the kept tile first, the one before it at the edge, as the
     * open row does. {@code high} is true when there is no open row above it, and it moves the whole
     * row up by three reference pixels -- 998 against 1001, 944 against 946 -- which are two
     * separately measured positions and not one number plus an offset, so both are written out.
     */
    static final class Next extends RefLayout {
        /** The row's y under an open row. {@code tv/browse/Rows.kt:164}. */
        static final int LOW_Y = 998;

        /** And under the hero or the bar, where it sits three higher. The same line. */
        static final int HIGH_Y = 1001;

        /** The row's name under an open row. {@code tv/browse/Rows.kt:169}. */
        static final int LOW_NAME_Y = 944;

        /** And under the hero or the bar. The same line. */
        static final int HIGH_NAME_Y = 946;

        private final TextView title;
        private final Cards.Poster behind;
        private final Cards.Poster[] posters = new Cards.Poster[NEXT_POSTERS];

        Next(Context context, Art.Source pictures) {
            super(context);
            // `.alpha(0.5f)` on the whole row at `Rows.kt:166`.
            setAlpha(0.5f);
            title = Type.line(context, 32, Type.BOLD, Color.WHITE);
            addView(title, RefLayout.at(Round.PAGE_EDGE, LOW_NAME_Y));
            behind = new Cards.Poster(context, pictures);
            addView(behind, RefLayout.at(Round.PAGE_EDGE - STEP, LOW_Y,
                    Cards.Poster.WIDE, Cards.Poster.HIGH));
            for (int slot = 0; slot < NEXT_POSTERS; slot++) {
                posters[slot] = new Cards.Poster(context, pictures);
                addView(posters[slot], RefLayout.at(Round.PAGE_EDGE + slot * STEP, LOW_Y,
                        Cards.Poster.WIDE, Cards.Poster.HIGH));
            }
        }

        void show(Feed.Row row, boolean high, int at) {
            if (row == null) {
                setVisibility(GONE);
                return;
            }
            setVisibility(VISIBLE);
            title.setText(row.title);
            Says.onto(title, Says.means(row.title));
            int top = high ? HIGH_Y : LOW_Y;
            move(title, Round.PAGE_EDGE, high ? HIGH_NAME_Y : LOW_NAME_Y);
            boolean hasBehind = at > 0;
            behind.setVisibility(hasBehind ? VISIBLE : GONE);
            if (hasBehind) {
                behind.show(row.at(at - 1));
                move(behind, Round.PAGE_EDGE - STEP, top);
            }
            for (int slot = 0; slot < NEXT_POSTERS; slot++) {
                Feed.Card card = row.at(at + slot);
                posters[slot].setVisibility(card == null ? GONE : VISIBLE);
                if (card != null) {
                    posters[slot].show(card);
                    move(posters[slot], Round.PAGE_EDGE + slot * STEP, top);
                }
            }
            requestLayout();
        }

        private void move(View child, int refX, int refY) {
            RefLayout.Params spot = (RefLayout.Params) child.getLayoutParams();
            spot.refX = refX;
            spot.refY = refY;
        }
    }
}
