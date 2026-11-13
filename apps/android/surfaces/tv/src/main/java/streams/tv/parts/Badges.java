package streams.tv.parts;

import android.content.Context;
import android.graphics.Color;
import android.view.Gravity;
import android.widget.LinearLayout;
import android.widget.TextView;
import streams.shared.Ink;
import streams.shared.Type;
import streams.tv.views.Ref;
import streams.tv.views.Round;

/**
 * The badges on a tile, as small pills at its bottom left. The Views twin of
 * {@code parts/Badge.kt:32-60}.
 *
 * <h2>It is handed words and never a badge</h2>
 * {@code Badge.kt:23-30} turns a {@code contract.Badge} into one of two words: the one for
 * {@code BadgeKind.NEW}, or the ranked one with {@code rank} filled into it. That is contract
 * reading, and it stays out of this package for a reason measured today rather than recalled:
 * {@code javap} over {@code apps/android/ports/build/classes/kotlin/main/contract/Badge.class} on
 * 2026-11-13 gives {@code public final java.lang.String getKind-vVTnwK0()}, and a hyphen is not a
 * legal character in a Java identifier. Note 240 closed half of that gate -- {@code Contract.java}
 * now compiles, so {@code contract.Contract$Badge} exists with a plain {@code getKind()} -- but it
 * is a different class from the {@code contract.Badge} that {@code ports.Seen} carries, and nothing
 * converts one into the other. So the words arrive already chosen, exactly as
 * {@code browse/Feed.java} takes them.
 *
 * <h2>Nothing here is said</h2>
 * No reading is put on this row or on the labels inside it, which is what the Compose path does
 * today: the poster applies {@code says(tile.name)} and the badges stay nodes of their own with
 * drawn text ({@code tv/app/Spoken.kt:41-42}).
 */
public final class Badges extends LinearLayout {
    /** The gap between two badges, and never before the first. {@code Badge.kt:40}. */
    static final int GAP = 8;

    /** A badge's padding, across and down. {@code Badge.kt:47}. */
    static final int PAD_ACROSS = 12;

    static final int PAD_DOWN = 4;

    /** The size of a badge's word. {@code Badge.kt:53}. */
    static final int TYPE = 22;

    /** How black the plate behind a badge is, in hundredths. {@code Badge.kt:46}. */
    static final int PLATE = 72;

    public Badges(Context context) {
        super(context);
        setOrientation(HORIZONTAL);
        setGravity(Gravity.CENTER_VERTICAL);
    }

    /**
     * Draws these words, or nothing at all when there are none -- {@code Badge.kt:34-35} returns
     * before it opens a row, so an empty badge list leaves no gap on the tile.
     */
    public void show(String[] said) {
        removeAllViews();
        if (said != null) {
            for (String word : said) {
                if (word != null && !word.isEmpty()) {
                    add(word);
                }
            }
        }
        setVisibility(getChildCount() == 0 ? GONE : VISIBLE);
    }

    private void add(String word) {
        Context context = getContext();
        TextView chip = Type.line(context, TYPE, Type.BOLD, Color.WHITE);
        chip.setText(word);
        chip.setBackground(Round.plate(context, Round.CHIP, Ink.black(PLATE)));
        chip.setPadding(Ref.px(context, PAD_ACROSS), Ref.px(context, PAD_DOWN),
                Ref.px(context, PAD_ACROSS), Ref.px(context, PAD_DOWN));
        chip.setGravity(Gravity.CENTER);
        LayoutParams spot = new LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT);
        if (getChildCount() > 0) {
            spot.leftMargin = Ref.px(context, GAP);
        }
        addView(chip, spot);
    }
}
