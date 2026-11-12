package streams.tv.views;

import android.content.Context;
import android.text.TextUtils;
import android.view.KeyEvent;
import android.view.View;
import android.view.ViewGroup;
import android.view.ViewTreeObserver;

/**
 * A television screen: the one node the toolkit focuses, laid out in reference coordinates, with
 * every key going through one funnel.
 *
 * The Views form of {@code Modifier.remote} ({@code tv/app/Remote.kt:71-95}) and of the box it is
 * applied to. 18 {@code .remote(} call sites across 17 screen files become 17 subclasses of this,
 * one per screen, each one building its children in code.
 *
 * <h2>Why a screen is one focusable node</h2>
 * Not a shortcut: it is this surface's design, written out at {@code Remote.kt:60-68} and
 * {@code tv/app/Spoken.kt:26-32}. A screen keeps where the remote is as plain state and paints it
 * (the open card draws its own border at {@code tv/browse/Rows.kt:110}), so there is no per-control
 * focus. Three consequences follow, and all three are settled here rather than at 17 call sites:
 *
 * <ul>
 * <li>{@code FOCUS_BLOCK_DESCENDANTS}, and this is the constant that matters.
 * {@code FOCUS_BEFORE_DESCENDANTS} -- the one a reader expects -- still lets a focusable child win
 * the focus, and then the keys arrive at the child and the screen stops moving. There are no
 * focusable children here, so blocking costs nothing and is the only constant that cannot be
 * defeated by one child added later with a background that happens to be focusable.
 * {@code FOCUS_BEFORE_DESCENDANTS} is right for exactly one kind of screen: one with a real text
 * field, which today is {@code tv/door/ServerAddress.kt} and which note 205 removes.</li>
 * <li><b>No {@code nextFocusLeft}/{@code Right}/{@code Up}/{@code Down}, anywhere.</b> Said in the
 * code because it is the obvious thing to reach for and it would be wrong: with one focusable node
 * there is no focus search to steer, and declaring the attributes would re-create the per-control
 * focus this surface deliberately does not have. They belong on a screen with a real field or a
 * real button, and not on this one.</li>
 * <li>{@code setDefaultFocusHighlightEnabled(false)}: the platform would draw its own focus ring
 * over a screen that paints its own, so a viewer would see two.</li>
 * </ul>
 *
 * <h2>And no Leanback</h2>
 * Nothing here is from {@code androidx.leanback}. Note 210 verified there is no Leanback library in
 * the tree at all -- the two hits are the platform manifest entries that put the app on a
 * television launcher -- so there is nothing to remove, only something not to add. What this uses is
 * {@code android.view.View}, {@code android.view.ViewGroup} and
 * {@code android.view.ViewTreeObserver}, all of which are older than Leanback and none of which is
 * deprecated.
 *
 * <h2>Getting the focus, which is the part Compose was hiding</h2>
 * {@code LaunchedEffect(live) { focus.requestFocus() }} ({@code Remote.kt:81-83}) ran after
 * composition, so it was never early. {@code requestFocus()} on a View fails before the view is
 * attached and laid out, and failing is silent: the screen draws and no key ever arrives. So three
 * paths, in this order, and each one is a case that really happens:
 * <ol>
 * <li>{@link #onAttachedToWindow()} posts the request. The normal case.</li>
 * <li>a one-shot global layout listener, for a screen added to a hierarchy that is already laid
 * out -- an overlay closing, a screen swapped for another.</li>
 * <li>{@link #onWindowFocusChanged(boolean)}, for the window coming back after a dialog or the
 * launcher took it, which note 214 recorded happening on these devices during its runs.</li>
 * </ol>
 * {@code android:focusedByDefault} and the {@code <requestFocus/>} tag are both unreliable on
 * television and are not used.
 */
public class RemoteHost extends RefLayout implements Funnel.Redraw {
    private final Funnel funnel = new Funnel(this::press, this);

    /** What a screen reader says the remote is on now, or empty for a screen that says nothing. */
    private String says = "";

    private ViewTreeObserver.OnGlobalLayoutListener waiting;

    public RemoteHost(Context context) {
        super(context);
        setFocusable(true);
        // The same APK runs on a phone ({@code app/Main.kt:29}), and without this a touch anywhere
        // takes the focus off this view and the keys stop arriving for the rest of the run.
        setFocusableInTouchMode(true);
        setDescendantFocusability(ViewGroup.FOCUS_BLOCK_DESCENDANTS);
        setDefaultFocusHighlightEnabled(false);
    }

    /**
     * What this screen does with a word. True when it used it.
     *
     * The default answers nothing, which is the honest default: a screen that has not implemented
     * its key map lets every key fall through to the system, rather than swallowing keys and
     * looking broken. Each screen overrides this with one call into its own state machine, and the
     * redraw is not its business -- see {@link #changed()}.
     */
    protected boolean press(Word word) {
        return false;
    }

    /**
     * Something moved, so draw. The one place a screen is told to redraw, called from
     * {@link Funnel} and from nowhere else.
     *
     * {@code invalidate()} and not {@code requestLayout()}: a screen that moves its focus paints
     * differently and lays out the same, because every part is at a measured coordinate. A screen
     * whose parts move overrides this.
     */
    @Override
    public void changed() {
        invalidate();
    }

    /** The funnel, for a screen that has to set its own state on it. */
    protected final Funnel funnel() {
        return funnel;
    }

    /**
     * Whether this screen holds the remote. False while something drawn over it holds the keys --
     * the opening, a profile menu, a PIN panel -- and true again after.
     *
     * One method replaces both the {@code on} parameter and {@code LocalCovered}
     * ({@code Remote.kt:76}, {@code Television.kt:59}), because to the keys a covered screen and a
     * screen that handed the remote to its own overlay are the same thing. The screen beneath keeps
     * all of its state, so it comes back where the viewer left it.
     */
    public final void setLive(boolean live) {
        if (funnel.isLive() == live) {
            return;
        }
        funnel.setLive(live);
        setFocusable(live);
        if (live) {
            requestFocus();
        }
    }

    public final boolean isLive() {
        return funnel.isLive();
    }

    /** True when this page runs right to left: the arrows swap and the layout mirrors. */
    public final void setRtl(boolean rtl) {
        funnel.setMirrored(rtl);
        setMirrored(rtl);
    }

    /**
     * What a screen reader says the remote is on now -- a tab's name, a title's name, a button's
     * word -- in the viewer's language, from {@code seen.words} or the catalogue, never written in
     * the code.
     *
     * This is a live region, so it is spoken again each time it changes, which is each time the
     * remote moves. Without it a person using a screen reader hears one thing when a screen opens
     * and nothing ever again, which is what {@code Remote.kt:86-91} is for.
     *
     * Empty leaves the description unset rather than setting it to empty, which is
     * {@code Says.kt:46-52}'s rule: a node described as nothing is worse than one a reader falls
     * through, and the words are empty for as long as a run has not read its language.
     *
     * The platform announces a live region when its description changes, so nothing is announced by
     * hand here; {@link #announce(String)} is the second channel, for something said that is not
     * this screen's own description.
     */
    public final void setSays(String says) {
        String words = says == null ? "" : says;
        if (words.equals(this.says)) {
            return;
        }
        this.says = words;
        if (words.isEmpty()) {
            setContentDescription(null);
            setAccessibilityLiveRegion(View.ACCESSIBILITY_LIVE_REGION_NONE);
        } else {
            setContentDescription(words);
            setAccessibilityLiveRegion(View.ACCESSIBILITY_LIVE_REGION_POLITE);
        }
    }

    public final String says() {
        return says;
    }

    /**
     * Something said once that is not this screen's own description: an answer arriving, a refusal,
     * a count that changed.
     *
     * A channel Compose does not expose, which {@code Spoken.kt:30-32} asks for. Nothing is said
     * for empty words.
     *
     * <b>It is deprecated at this compileSdk and is kept anyway.</b> Verified 2026-11-12 by
     * compiling this file with {@code -Xlint:deprecation} against
     * {@code platforms/android-37.0/android.jar}: "announceForAccessibility(CharSequence) in View
     * has been deprecated". Note 210 names this call as one of the things Views makes easier, and
     * that is still true, but the platform now points at a live region instead -- which
     * {@link #setSays(String)} already sets, so the ordinary case does not come through here. What
     * is left is the one-off that is not a description: kept, with the warning suppressed in one
     * place rather than at every screen, and named in the report as the owner's or the lead's call
     * to drop.
     */
    @SuppressWarnings("deprecation")
    public final void announce(String words) {
        if (!TextUtils.isEmpty(words)) {
            announceForAccessibility(words);
        }
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        // `dispatchKeyEvent` and not `onKeyDown`: this is the Views call with the property
        // `onPreviewKeyEvent` had, which is that it sees the key before any descendant.
        // `onKeyDown` arrives only after the focused view has refused it.
        if (funnel.key(event.getKeyCode(), event.getAction())) {
            return true;
        }
        return super.dispatchKeyEvent(event);
    }

    @Override
    protected void onAttachedToWindow() {
        super.onAttachedToWindow();
        post(this::take);
        waiting = new ViewTreeObserver.OnGlobalLayoutListener() {
            @Override
            public void onGlobalLayout() {
                stopWaiting();
                take();
            }
        };
        getViewTreeObserver().addOnGlobalLayoutListener(waiting);
    }

    @Override
    protected void onDetachedFromWindow() {
        stopWaiting();
        super.onDetachedFromWindow();
    }

    @Override
    public void onWindowFocusChanged(boolean hasWindowFocus) {
        super.onWindowFocusChanged(hasWindowFocus);
        if (hasWindowFocus) {
            take();
        }
    }

    private void take() {
        if (funnel.isLive() && !hasFocus()) {
            requestFocus();
        }
    }

    private void stopWaiting() {
        if (waiting != null) {
            getViewTreeObserver().removeOnGlobalLayoutListener(waiting);
            waiting = null;
        }
    }
}
