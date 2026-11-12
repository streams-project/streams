package streams.tv.browse;

import android.content.Context;
import android.graphics.Color;
import android.view.Gravity;
import android.view.View;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import streams.shared.R;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Round;
import streams.tv.views.Says;
import streams.tv.views.Shade;
import streams.tv.views.Type;
import streams.tv.views.Words;

/**
 * The rounded card at the top of a page. {@code tv/browse/HeroCard.kt:46-99}.
 *
 * <h2>The width is written as the derivation and not as the answer</h2>
 * {@code (1920 - 2 * PAGE_EDGE)} at {@code HeroCard.kt:54}, and {@code NumbersTest.kt}'s sixth test
 * exists to keep it written that way: "a rewrite that writes the answer loses the reason, and the
 * next person to change the margin finds the hero does not follow". 888 dp is what it comes to and
 * 888 is not what is written. So the Java keeps the same expression, over {@link streams.tv.views.Ref#W}
 * and {@link Round#PAGE_EDGE}, and the test holds the two sides equal.
 *
 * <h2>What a screen reader is handed</h2>
 * Three readings, which are {@code HeroCard.kt:61-63}'s three: the artwork behind the card is the
 * title that is named, pictured and described on top of it, so it is decoration, and the two shades
 * over it are nothing at all. All three go away with their subtrees rather than becoming silent
 * stops. {@code SpokenTest.kt}'s census records this file at 3 calls, and these are the three.
 */
final class HeroView extends RefLayout implements Words.Bound {
    /** The hero's own y. {@code tv/browse/HeroCard.kt:53}. */
    static final int Y = 143;

    /** And its height, the next line. */
    static final int HIGH = 767;

    /** The words inside it sit on this column and this foot. {@code tv/browse/HeroCard.kt:64}. */
    static final int INSET = 30;

    /** {@code tv/browse/HeroCard.kt:64}. */
    static final int FOOT = 34;

    /** And they are this wide. The same line. */
    static final int COLUMN = 760;

    private final Art art;
    private final View side;
    private final View shade;
    private final LinearLayout column;
    private final ImageView logo;
    private final TextView name;
    private final TextView story;
    private final LinearLayout buttons;
    private final Parts.Pill play;
    private final Parts.Pill more;
    private View facts;

    HeroView(Context context, Art.Source pictures) {
        super(context);
        setClipChildren(true);
        setClipToOutline(true);
        // `HeroCard.kt:56-57`: the plate, and a white border at a quarter strength on the panel's
        // own corner.
        setBackground(Round.edged(context, Round.PANEL, Ink.PLATE, 2, Ink.white(28)));

        art = new Art(context, pictures, null);
        addView(art, RefLayout.at(0, 0, Ref.W - 2 * Round.PAGE_EDGE, HIGH));

        // `HeroCard.kt:43`: dark on the side the words start from, fading out by 0.62 across.
        // `fromStart` (`app/Round.kt:66-71`) turns the gradient round for a right to left page; here
        // the orientation is read off the one mirror switch the whole layout has.
        side = new View(context);
        Says.onto(side, Says.decoration());
        addView(side, RefLayout.at(0, 0, Ref.W - 2 * Round.PAGE_EDGE, HIGH));
        dress();

        // `HeroCard.kt:44`: and a second shade up the card, from 0.55 of its height.
        shade = new View(context);
        shade.setBackground(Shade.down(
                new int[] {Color.TRANSPARENT, Ink.black(55)}, new float[] {0.55f, 1f}));
        Says.onto(shade, Says.decoration());
        addView(shade, RefLayout.at(0, 0, Ref.W - 2 * Round.PAGE_EDGE, HIGH));

        // The one nested box on this page that is a stack and not a set of coordinates, because
        // `HeroCard.kt:64-96` is a Column whose parts push each other down: a title with a logo is
        // a different height from one without, and the facts line is as tall as its words. The
        // three gaps between them are the measured numbers at `:80`, `:82` and `:92`.
        column = new LinearLayout(context);
        column.setOrientation(LinearLayout.VERTICAL);

        logo = new ImageView(context);
        logo.setScaleType(ImageView.ScaleType.FIT_START);
        logo.setAdjustViewBounds(true);
        logo.setMaxWidth(Ref.px(context, 600));
        logo.setMaxHeight(Ref.px(context, 200));
        column.addView(logo, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT));

        name = Type.story(context, 76, 80, 2, Color.WHITE);
        column.addView(name);

        story = Type.story(context, 30, 39, 3, Ink.white(80));
        column.addView(story, gap(context, 16));

        buttons = new LinearLayout(context);
        buttons.setOrientation(LinearLayout.HORIZONTAL);
        buttons.setGravity(Gravity.CENTER_VERTICAL);
        play = new Parts.Pill(context, "common.play",
                context.getDrawable(R.drawable.icon_play));
        more = new Parts.Pill(context, "common.more_info", null);
        buttons.addView(play, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT, Ref.px(context, 84)));
        LinearLayout.LayoutParams beside = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT, Ref.px(context, 84));
        // `HeroCard.kt:93`: the two buttons sit 16 apart.
        beside.leftMargin = Ref.px(context, 16);
        buttons.addView(more, beside);
        column.addView(buttons, gap(context, 30));

        addView(column, RefLayout.at(INSET, 0, COLUMN, RefLayout.WRAP));
    }

    /** A child with a measured gap above it: {@code Spacer(Modifier.height(n.ref))}. */
    private static LinearLayout.LayoutParams gap(Context context, int ref) {
        LinearLayout.LayoutParams spot = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        spot.topMargin = Ref.px(context, ref);
        return spot;
    }

    /**
     * Draws the hero for a title.
     *
     * @param button 0 for Play, 1 for More info, or -1 when the remote is somewhere else on the
     *               page, which is {@code Browse.kt:288}'s own three answers
     */
    void show(Feed.Card lead, Feed.Story told, int button) {
        art.show(lead == null ? "" : lead.wide, Ref.W - 2 * Round.PAGE_EDGE, HIGH);
        String picture = told == null ? "" : told.logo;
        // `HeroCard.kt:68-79`: the logo is the title's name drawn as a picture, so it says the name
        // and the written name is not drawn as well. Without one, the name is text.
        boolean hasLogo = !picture.isEmpty();
        logo.setVisibility(hasLogo ? VISIBLE : GONE);
        name.setVisibility(hasLogo ? GONE : VISIBLE);
        name.setText(lead == null ? "" : lead.name);
        if (hasLogo) {
            Says.onto(logo, Says.says(lead == null ? "" : lead.name));
        }
        if (facts != null) {
            column.removeView(facts);
            facts = null;
        }
        if (told != null && told.facts.length > 0) {
            facts = Parts.facts(getContext(), told.facts);
            // `HeroCard.kt:80-81`: 16 above the facts, and `:82` another 16 below them, which is
            // the gap already on the story.
            column.addView(facts, 2, gap(getContext(), 16));
        }
        story.setText(told == null ? "" : told.overview);
        play.lit(button == 0);
        more.lit(button == 1);
    }

    /** The two buttons' words, read again when the language changes. */
    @Override
    public void words(Words words) {
        play.words(words);
        more.words(words);
    }

    @Override
    protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
        super.onLayout(changed, left, top, right, bottom);
        // `HeroCard.kt:64`: the column is aligned to the card's bottom and inset from its start, so
        // its top is the card's foot less however tall the column measured.
        int inset = Ref.px(this, INSET);
        int y = getHeight() - Ref.px(this, FOOT) - column.getMeasuredHeight();
        int x = isMirrored() ? getWidth() - inset - column.getMeasuredWidth() : inset;
        column.layout(x, y, x + column.getMeasuredWidth(), y + column.getMeasuredHeight());
    }

    /**
     * The side shade starts dark on the side the words start from and is gone by 0.62 across.
     * {@code HeroCard.kt:43}, through {@code fromStart} at {@code app/Round.kt:66-71}: it is the one
     * shade on this page that turns round for a right to left language, so it is rebuilt whenever
     * the mirror changes rather than read from a layout direction at draw time.
     */
    private void dress() {
        side.setBackground(Shade.across(new int[] {Ink.black(80), Color.TRANSPARENT},
                new float[] {0f, 0.62f}, isMirrored()));
    }

    /**
     * Mirrors this card and turns its side shade round with it.
     *
     * Not an override: {@code RefLayout.setMirrored} is final, deliberately, so that the mirror is
     * one switch nobody can intercept. So the two things that have to happen together are named
     * here and the page calls this instead.
     */
    void mirror(boolean mirrored) {
        setMirrored(mirrored);
        dress();
    }
}
