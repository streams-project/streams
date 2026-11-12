package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import org.junit.Test;

/**
 * Where a row opens when the viewer comes back to it.
 *
 * The behaviour is note 187's, and the defect it fixed was the owner's own report from the
 * simulator: after scrolling a row and moving up or down, the row showed from its first tile as if
 * it had never been scrolled. The fix (Android {@code fd7b104}) keeps each row's place by the row's
 * id together with the title that was on it, so a fresh answer that reorders a row follows the
 * title. These hold that behaviour, not the code that implements it.
 *
 * The row is handed as its title keys, which is why these are unit tests: no contract type, no
 * toolkit, no device. Note 210 section 4 measured that 184 of the generated getters carry a hyphen
 * in their JVM name and so cannot be called from Java source at all until wave 0 lands, so a kernel
 * written against the contract's own types could not be tested today -- or compiled.
 */
public class RowMemoryTest {
    private static final String ROW = "row:picked-for-you";
    private static final String OTHER = "row:because-you-watched";
    private static final String[] TILES = {"movie:1", "movie:2", "movie:3", "movie:4", "movie:5"};

    /** A row nobody has been on opens at its first tile. */
    @Test
    public void a_row_nobody_has_been_on_opens_at_its_first_tile() {
        RowMemory memory = new RowMemory();
        assertFalse(memory.knows(ROW));
        assertEquals(0, memory.columnOf(ROW, TILES));
    }

    /** A row the viewer comes back to opens where they left it. */
    @Test
    public void a_row_opens_where_the_viewer_left_it() {
        RowMemory memory = new RowMemory();
        memory.keep(ROW, TILES, 3);
        assertTrue(memory.knows(ROW));
        assertEquals(3, memory.columnOf(ROW, TILES));
    }

    /**
     * A fresh answer that moved the title follows the title.
     *
     * This is the half of note 187's fix that the place alone does not give: returning to a page
     * fetches it again and the server may reorder the tiles, so the place the viewer was on is the
     * title they were on, not the index.
     */
    @Test
    public void a_fresh_answer_that_moved_the_title_follows_the_title() {
        RowMemory memory = new RowMemory();
        memory.keep(ROW, TILES, 3);
        String[] reordered = {"movie:9", "movie:4", "movie:1", "movie:2", "movie:3"};
        assertEquals(1, memory.columnOf(ROW, reordered));
    }

    /** A row whose title has gone altogether keeps the place, clamped inside the row. */
    @Test
    public void a_row_whose_title_has_gone_keeps_the_place_inside_the_row() {
        RowMemory memory = new RowMemory();
        memory.keep(ROW, TILES, 4);
        String[] fresh = {"movie:7", "movie:8", "movie:9"};
        assertEquals(2, memory.columnOf(ROW, fresh));
    }

    /** And a row that has emptied opens at nothing rather than outside itself. */
    @Test
    public void a_row_that_has_emptied_opens_at_nothing() {
        RowMemory memory = new RowMemory();
        memory.keep(ROW, TILES, 4);
        assertEquals(0, memory.columnOf(ROW, new String[0]));
    }

    /**
     * Two rows keep two places, because the place is kept under the row's id.
     *
     * The defect this is about: places kept by row number alone, so a page whose rows the server
     * reordered moved every row's place at once. Note 187 records both halves -- the row sliding
     * away was drawn at tile 0, and places were kept by row number.
     */
    @Test
    public void two_rows_keep_two_places() {
        RowMemory memory = new RowMemory();
        memory.keep(ROW, TILES, 3);
        memory.keep(OTHER, TILES, 1);
        assertEquals(3, memory.columnOf(ROW, TILES));
        assertEquals(1, memory.columnOf(OTHER, TILES));
    }

    /** Moving along a row keeps the new place, which is what {@code Browse.kt:100-103} writes on every step. */
    @Test
    public void moving_along_a_row_keeps_the_new_place() {
        RowMemory memory = new RowMemory();
        for (int at = 0; at < TILES.length; at++) {
            memory.keep(ROW, TILES, at);
            assertEquals(at, memory.columnOf(ROW, TILES));
        }
    }

    /**
     * A kept place outside the row is a bounds answer and not a focus answer.
     *
     * {@code Browse.kt:93} compares the title at the kept place with the kept title, and two nulls
     * compare equal, so a kept place outside a row with a null title comes back as a place the row
     * does not have. Unreachable today, and a crash if it were reached. This states the Java
     * answers the clamp instead.
     */
    @Test
    public void a_kept_place_outside_the_row_is_clamped_and_not_returned() {
        RowMemory memory = new RowMemory();
        memory.keep(ROW, new String[0], 7);
        assertEquals(0, memory.columnOf(ROW, new String[0]));
        assertEquals(2, memory.columnOf(ROW, new String[] {"movie:1", "movie:2", "movie:3"}));
    }

    /** A page whose answer is not its own forgets everything. */
    @Test
    public void forgetting_a_page_forgets_every_row_of_it() {
        RowMemory memory = new RowMemory();
        memory.keep(ROW, TILES, 3);
        memory.clear();
        assertFalse(memory.knows(ROW));
        assertEquals(0, memory.columnOf(ROW, TILES));
    }
}
