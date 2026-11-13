package streams.shared;

import java.util.HashMap;
import java.util.Map;

/**
 * Where the viewer was on each page, for the run of the app.
 *
 * The Views form of {@code LocalPages} ({@code tv/browse/Browse.kt:58}, held for the run at
 * {@code tv/app/Television.kt:39}, keyed at {@code Browse.kt:164}). Simpler here than in Compose
 * because there is no composition local to thread through: one of these is a field on the activity
 * or on the root, handed to whatever screen asks.
 *
 * <h2>Why this one is in the shared module and its neighbours are not</h2>
 * It was written for the television in {@code tv/.../views} and moved here on 2026-11-13, because
 * nothing in it is a television. There is no remote in it ({@link java.util.Map} of
 * {@link String} to {@link Object} and nothing else), no reference pixel, no view and no context,
 * so a Java screen on the phone holds its pages with the same class rather than with a second copy
 * of it. The three parts of {@link #page} -- who is watching, which screen, which genre -- are the
 * product's idea of a page and not the television's; what is television-shaped is the exact
 * rendering of the two edge cases below, which is kept because the Kotlin it replaces builds the
 * key that way and a key that differed would lose the viewer's place on return.
 *
 * What it holds, today, is a page's focus state and its growth past the first twelve rows
 * ({@code Browse.kt:165}, {@code :176-178}). It is deliberately a map of {@link Object}: the pieces
 * it keeps belong to the screens, which are wave 2's, and a kernel that named their types would
 * have to be edited by every one of them.
 */
public final class PageMemory {
    private final Map<String, Object> held = new HashMap<>();

    /**
     * What a page is called, for keeping and finding what the viewer left on it.
     *
     * An exact port of {@code Browse.kt:164}: {@code "${seen.active} $current ${genre.orEmpty()}"}.
     * Exact including the two things a reader would tidy up, because tidying either one splits the
     * memory in half and the viewer loses their place on return:
     * <ul>
     * <li>a null profile renders as the four letters a Kotlin template renders it as, so the key
     * before a profile is chosen is the same key either language builds;</li>
     * <li>a missing genre renders as nothing at all, so the key keeps its trailing space.</li>
     * </ul>
     *
     * @param active  the profile in use, {@code seen.active}
     * @param screen  the page, {@code seen.route?.screen?.value} or the default
     * @param genre   the genre inside it, or null
     */
    public static String page(String active, String screen, String genre) {
        return String.valueOf(active) + " " + screen + " " + (genre == null ? "" : genre);
    }

    /** What was kept for this page, or null when the viewer has not been on it this run. */
    public Object get(String page) {
        return held.get(page);
    }

    /** Keeps this page's state for the run of the app. */
    public void put(String page, Object state) {
        held.put(page, state);
    }

    /**
     * What was kept for this page, or what {@code maker} builds, kept and returned.
     * {@code getOrPut} at {@code Browse.kt:165}.
     */
    public Object getOrPut(String page, Maker maker) {
        Object found = held.get(page);
        if (found == null) {
            found = maker.make();
            held.put(page, found);
        }
        return found;
    }

    /** What builds a page's state the first time the viewer arrives on it. */
    public interface Maker {
        Object make();
    }

    /** How many pages are held, for a test to say that one page is one entry. */
    public int size() {
        return held.size();
    }
}
