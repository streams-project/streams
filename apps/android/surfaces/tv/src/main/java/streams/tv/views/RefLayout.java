package streams.tv.views;

import android.content.Context;
import android.util.AttributeSet;
import android.view.View;
import android.view.ViewGroup;

/**
 * The television's layout: children placed by absolute reference coordinates, one level deep.
 *
 * <h2>Why this shape</h2>
 * This surface already draws in a fixed reference space. Measured on 2026-11-12 (note 210, section
 * 1): 106 {@code .offset(} sites and 347 {@code n.ref} uses in {@code tv/src/main/kotlin}, with the
 * canvas width written out as {@code 1920} at {@code tv/browse/HeroCard.kt:54} and
 * {@code tv/browse/TopBar.kt:117}. There is no flow, no stack and no constraint in it -- a part
 * knows the coordinate it was measured at. So the whole surface needs one {@link ViewGroup} whose
 * layout parameters are that coordinate, and the 167 nested {@code Box}/{@code Row}/{@code Column}
 * calls become children of it.
 *
 * <h2>Why flat, and why this is the part that keeps the win</h2>
 * Note 214 measured that Compose's {@code setContent} is CHEAPER than inflating XML -- 9.0 ms
 * against 22.0 on a Google TV Streamer, 7.0 against 14.5 on a Pixel 8 -- so the Views advantage is
 * not in getting the first screen built, it is in measure, layout and draw: {@code :kc} spent
 * 150.0 ms in measure and layout where {@code :jv} spent 75.5 for the same 20 posters, and
 * {@code :ce} spent 51.0 with an empty Box. A deep hierarchy hands that straight back, because
 * every level is another measure pass over the same children. Hence: one of these per screen,
 * children placed, nothing nested that does not have to be.
 *
 * {@link #onMeasure} is a single loop with no second pass, so a screen costs one measure per child
 * and not one per child per level.
 *
 * <h2>Right to left</h2>
 * One line, in one place, instead of 106 call sites reading a layout direction: see
 * {@link #leftOf}. The switch is {@link #setMirrored(boolean)}, fed from {@code seen.rtl} as
 * {@code tv/app/Television.kt:52} feeds it today. A wrong sign there mirrors every screen at once,
 * which is why {@link #leftOf} is a static function with its own tests rather than three lines
 * inside {@link #onLayout}.
 *
 * <h2>No recycling</h2>
 * None of this needs a {@code RecyclerView}. The television surface has zero lazy lists (note 210,
 * decision 1: {@code LazyColumn|LazyRow|LazyVerticalGrid} returns 0 matches over
 * {@code tv/src/main/kotlin}; the phone returns 12), and the widest row holds one open card plus
 * {@code AHEAD = 5} posters ahead and one behind, with {@code NEXT_POSTERS = 6} on the dim row
 * below ({@code tv/browse/Rows.kt:44-46}). At most a dozen children, every one of them on screen.
 */
public class RefLayout extends ViewGroup {
    /** A child that measures itself rather than being given a reference size. */
    public static final int WRAP = -1;

    /** True when the interface mirrors, which is the whole of right to left in this layout. */
    private boolean mirrored;

    /**
     * Children are not clipped to this layout, because a Compose {@code Box} does not clip its
     * children either and this surface relies on that: a row slides its posters in from past the
     * edge of the screen, and a card's focus border is drawn on the outside of it. Clipping is asked
     * for per part, by the part that wants it.
     */
    public RefLayout(Context context) {
        super(context);
        setClipChildren(false);
    }

    public RefLayout(Context context, AttributeSet attributes) {
        super(context, attributes);
        setClipChildren(false);
    }

    /**
     * Where a child's left edge goes, in pixels.
     *
     * Two formulas, and the difference between them matters by a pixel. When the child declares its
     * reference width the mirror is taken in reference space -- {@code W - refX - refW}, note 210's
     * own form -- so the mirrored position is one conversion of one integer and cannot drift from
     * the unmirrored one. When the child measures itself ({@link #WRAP}) there is no reference width
     * to subtract, so the mirror is taken in pixels against the parent's own width, and that is a
     * subtraction of three separately rounded numbers.
     *
     * The second one lands within a pixel of the first and not on it, measured in
     * {@code RefLayoutTest}: a reference pixel is half a dp, so an odd remainder mirrors to a half
     * dp, and at density 1 that is half a physical pixel the subtraction has nowhere to put. So a
     * mirrored child should declare its reference width wherever it has one, and a line of text is
     * the case where it does not.
     *
     * @param refX        the child's x as it was measured, in reference pixels
     * @param refW        its measured width, or {@link #WRAP}
     * @param childWidth  what it measured to, in pixels
     * @param parentWidth this layout's width, in pixels
     * @param mirrored    true for a right to left interface
     * @param density     the screen's density
     */
    public static int leftOf(int refX, int refW, int childWidth, int parentWidth, boolean mirrored, float density) {
        if (!mirrored) {
            return Ref.px(refX, density);
        }
        if (refW != WRAP) {
            return Ref.px(Ref.W - refX - refW, density);
        }
        return parentWidth - Ref.px(refX, density) - childWidth;
    }

    /**
     * And where its top edge goes. There is no mirror down the screen in any language this project
     * speaks, so this is the conversion and nothing else. It is a function for one reason: so a
     * reader of {@link #onLayout} sees the asymmetry between the two axes stated rather than
     * implied.
     */
    public static int topOf(int refY, float density) {
        return Ref.px(refY, density);
    }

    /** True when this layout places its children from the right. */
    public final boolean isMirrored() {
        return mirrored;
    }

    /** Mirrors every child of this layout, which is the whole of right to left here. */
    public final void setMirrored(boolean mirrored) {
        if (this.mirrored != mirrored) {
            this.mirrored = mirrored;
            requestLayout();
        }
    }

    /** A child at a measured coordinate, with a measured size. */
    public static Params at(int refX, int refY, int refW, int refH) {
        return new Params(refX, refY, refW, refH);
    }

    /** A child at a measured coordinate that measures its own size: a line of text, a label. */
    public static Params at(int refX, int refY) {
        return new Params(refX, refY, WRAP, WRAP);
    }

    @Override
    protected void onMeasure(int widthSpec, int heightSpec) {
        int width = MeasureSpec.getSize(widthSpec);
        int height = MeasureSpec.getSize(heightSpec);
        float density = Ref.density(this);
        for (int index = 0; index < getChildCount(); index++) {
            View child = getChildAt(index);
            if (child.getVisibility() == GONE) {
                continue;
            }
            Params spot = paramsOf(child);
            child.measure(
                    specOf(spot.refW, width - Ref.px(spot.refX, density), density),
                    specOf(spot.refH, height - Ref.px(spot.refY, density), density));
        }
        setMeasuredDimension(width, height);
    }

    @Override
    protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
        int width = right - left;
        float density = Ref.density(this);
        for (int index = 0; index < getChildCount(); index++) {
            View child = getChildAt(index);
            if (child.getVisibility() == GONE) {
                continue;
            }
            Params spot = paramsOf(child);
            int x = leftOf(spot.refX, spot.refW, child.getMeasuredWidth(), width, mirrored, density);
            int y = topOf(spot.refY, density);
            child.layout(x, y, x + child.getMeasuredWidth(), y + child.getMeasuredHeight());
        }
    }

    /**
     * A measure specification for one axis of one child: exact when the size was measured off a
     * capture, and at most what is left of the parent when the child measures itself.
     *
     * {@code AT_MOST} and not {@code UNSPECIFIED} for the wrapping case, because a line of text
     * given no bound lays itself out on one line however long it is, which is how a name runs off
     * the side of a screen.
     */
    private static int specOf(int ref, int room, float density) {
        if (ref != WRAP) {
            return MeasureSpec.makeMeasureSpec(Ref.px(ref, density), MeasureSpec.EXACTLY);
        }
        return MeasureSpec.makeMeasureSpec(Math.max(room, 0), MeasureSpec.AT_MOST);
    }

    private static Params paramsOf(View child) {
        LayoutParams params = child.getLayoutParams();
        if (params instanceof Params) {
            return (Params) params;
        }
        // A child added with no coordinate sits at the origin at its own size, which is what a
        // caller that forgot the parameters sees on screen: a part in the corner, not a crash.
        return new Params(0, 0, WRAP, WRAP);
    }

    @Override
    protected LayoutParams generateDefaultLayoutParams() {
        return new Params(0, 0, WRAP, WRAP);
    }

    @Override
    protected boolean checkLayoutParams(LayoutParams params) {
        return params instanceof Params;
    }

    @Override
    protected LayoutParams generateLayoutParams(LayoutParams from) {
        return from instanceof Params ? from : new Params(0, 0, WRAP, WRAP);
    }

    /**
     * Where a part was measured, in the reference space the captures are in.
     *
     * The four numbers are the measurement itself. They are held as reference pixels and never as
     * dp or as pixels, so a call site reads {@code at(72, 200, 904, 508)} -- the four numbers at
     * {@code tv/browse/Rows.kt:87-88} -- and the conversion happens once, here.
     */
    public static class Params extends ViewGroup.LayoutParams {
        public int refX;
        public int refY;
        public int refW;
        public int refH;

        public Params(int refX, int refY, int refW, int refH) {
            super(WRAP_CONTENT, WRAP_CONTENT);
            this.refX = refX;
            this.refY = refY;
            this.refW = refW;
            this.refH = refH;
        }
    }
}
