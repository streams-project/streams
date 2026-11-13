package streams.phone;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotSame;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertSame;

import org.junit.Test;
import streams.shared.PageMemory;

/**
 * That the phone can hold its pages in the shared memory, and that the memory holds the phone's
 * own kind of state without naming it.
 *
 * <h2>Why this file is Java and why it is here</h2>
 * The Java foundation of note 210's wave 1 landed in
 * {@code surfaces/tv/src/main/java/streams/tv/views/} and {@code surfaces/shared} had no Java in it
 * at all, so nothing the phone compiles could reach any of it and wave 4 was blocked before it
 * started (note 235). {@link PageMemory} moved to {@code surfaces/shared} on 2026-11-13, and this
 * file is the proof that the move did what it was for: it is Java, it is in the phone's own test
 * source set, and it names a class the phone's module does not hold. If the class were still in the
 * television's folder, or if an Android library module could not compile Java in
 * {@code src/main/java}, this file would not compile and the module's tests would not run.
 *
 * <h2>What is asserted, which is not what the television's test asserts</h2>
 * {@code tv/.../views/PageMemoryTest.java} pins the shape of the key against the Kotlin it was
 * ported from. That is the television's question. The phone's question is the other half: the
 * memory is a map of {@link Object} on purpose, so a surface keeps its own kind of state in it --
 * on the phone a list's first visible row, which is nothing the television has -- and two pages
 * keep two of them.
 */
public class PagesTest {
    /** Where a scrolling page was left. The phone's own state, of a type the memory never names. */
    private static final class Scrolled {
        int row;
    }

    /** A page the viewer has not opened this run holds nothing, so a caller builds its state. */
    @Test
    public void a_page_not_yet_opened_holds_nothing() {
        PageMemory pages = new PageMemory();
        assertNull(pages.get(PageMemory.page("yesterday", "home", null)));
        assertEquals(0, pages.size());
    }

    /**
     * A page's state is built on the first visit and found again on the next, which is what keeps
     * a viewer's place when they open a title and come back.
     */
    @Test
    public void a_page_keeps_the_state_the_phone_put_in_it() {
        PageMemory pages = new PageMemory();
        String home = PageMemory.page("yesterday", "home", null);
        Scrolled first = (Scrolled) pages.getOrPut(home, Scrolled::new);
        first.row = 7;
        Scrolled again = (Scrolled) pages.getOrPut(home, Scrolled::new);
        assertSame("the second visit finds the first visit's state", first, again);
        assertEquals("and the row it was left on", 7, again.row);
        assertEquals(1, pages.size());
    }

    /** Two pages are two places, so one page's scroll is not the other's. */
    @Test
    public void two_pages_keep_two_places() {
        PageMemory pages = new PageMemory();
        Scrolled home = (Scrolled) pages.getOrPut(PageMemory.page("yesterday", "home", null),
                Scrolled::new);
        Scrolled comedy = (Scrolled) pages.getOrPut(PageMemory.page("yesterday", "home", "comedy"),
                Scrolled::new);
        assertNotSame(home, comedy);
        assertEquals(2, pages.size());
    }
}
