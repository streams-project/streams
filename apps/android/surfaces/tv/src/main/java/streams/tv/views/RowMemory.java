package streams.tv.views;

import java.util.HashMap;
import java.util.Map;
import java.util.Objects;

/**
 * Where each row of a page was left: the place, and the title that was on it.
 *
 * A line-for-line port of {@code PageFocus.columns}, {@code columnOf} and {@code keep} at
 * {@code tv/browse/Browse.kt:86-103}, which is the fix note 187 records as Android {@code fd7b104}.
 * The defect it fixed, in the owner's words from the simulator: after scrolling a row and moving up
 * or down, the row showed from its first tile as if it had never been scrolled.
 *
 * <h2>Why the title and not just the place</h2>
 * Returning to a page fetches it again and the server may reorder its rows and the tiles inside
 * them (note 187, "Found while checking"). A place kept by number alone then opens a row on a
 * different title. So a row's place is kept by the row's id together with the title that was on it,
 * and a fresh answer that moved the title follows the title.
 *
 * <h2>Why it takes an array of title keys and not the contract's types</h2>
 * Deliberate, and it is what lets this class exist today. Note 210 section 4 measured that 69 of
 * the generated {@code contract.*} classes carry name-mangled getters -- 184 of them -- because the
 * Kotlin emitter writes each open word type as a {@code @JvmInline value class}, and a hyphen is
 * not a legal character in a Java identifier. So {@code tile.getTitle()} cannot be called from Java
 * source until wave 0's package S1 lands. A kernel written against {@code List<Tile>} would be a
 * kernel that does not compile. This one is handed the row's title keys as strings by whoever has
 * them, so the focus memory is finished and testable before the contract work is, and it keeps no
 * contract type when that work does land.
 *
 * It holds no toolkit either: no view, no context, no snapshot state. That is what makes a scripted
 * key sequence a unit test rather than a device session.
 */
public final class RowMemory {
    /** A place and the title that was on it. */
    private static final class Kept {
        final int at;
        final String title;

        Kept(int at, String title) {
            this.at = at;
            this.title = title;
        }
    }

    private final Map<String, Kept> columns = new HashMap<>();

    /**
     * The tile a row was left on, or its first.
     *
     * The three answers of {@code Browse.kt:89-96}, in order: the kept place when the title is
     * still there; where that title moved to when a fresh answer moved it; and the kept place
     * clamped into the row when the title has gone from the row altogether.
     *
     * One difference from the Kotlin, stated rather than hidden. {@code Browse.kt:93} reads
     * {@code tiles.getOrNull(at)?.title == title}, and when the kept place is outside the row and
     * the kept title is null those two nulls compare equal, so it returns a place the row does not
     * have. That is unreachable today -- {@code keep} is only called from inside a row, so the
     * title it stores is never null -- but it is a bounds answer and not a focus answer, so this
     * checks the bounds first and falls through to the clamp. A port that carried the hole across
     * would be a port of the hole.
     *
     * @param id     the row's own id, which is what the place is kept under
     * @param titles the row's title keys, in the order it draws them
     */
    public int columnOf(String id, String[] titles) {
        Kept kept = columns.get(id);
        if (kept == null) {
            return 0;
        }
        int last = titles.length - 1;
        if (kept.at >= 0 && kept.at <= last && Objects.equals(titles[kept.at], kept.title)) {
            return kept.at;
        }
        for (int at = 0; at <= last; at++) {
            if (Objects.equals(titles[at], kept.title)) {
                return at;
            }
        }
        return Math.min(Math.max(kept.at, 0), Math.max(last, 0));
    }

    /** Keeps a row's place and the title on it, which is both halves of {@code Browse.kt:100-103}. */
    public void keep(String id, String[] titles, int at) {
        String title = at >= 0 && at < titles.length ? titles[at] : null;
        columns.put(id, new Kept(at, title));
    }

    /** True when this row has a kept place, which is not the same as its place being 0. */
    public boolean knows(String id) {
        return columns.containsKey(id);
    }

    /** Forgets everything, for a page whose answer is not its own. */
    public void clear() {
        columns.clear();
    }
}
