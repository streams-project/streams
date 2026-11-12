package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotEquals;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertSame;

import java.util.regex.Matcher;
import org.junit.Test;

/**
 * What a page is called, and that one page is one place in the memory.
 *
 * The key is the whole of this: {@code LocalPages} holds a page's focus state for the run of the app
 * ({@code tv/browse/Browse.kt:58}, {@code tv/app/Television.kt:39}), so two pages that build the
 * same key share one state and a viewer's place jumps between them, while one page that builds two
 * keys loses its place on every visit. Neither failure throws anything.
 */
public class PageMemoryTest {
    /**
     * The key is the three parts {@code Browse.kt:164} joins, in that order, with the spaces.
     *
     * The form is read out of the Kotlin and not copied here, so a page key edited on one side and
     * not the other fails this rather than being discovered by a viewer who lost their place.
     */
    @Test
    public void the_page_key_is_the_three_parts_the_kotlin_joins() {
        Matcher said = Repo.theOne("val page = \"([^\"]*)\"", 0, Repo.text(BROWSE),
                "the page key in " + BROWSE);
        assertEquals("the key is the profile, the page and the genre, in that order",
                "${seen.active} $current ${seen.route?.genre.orEmpty()}", said.group(1));
        assertEquals("yesterday home ", PageMemory.page("yesterday", "home", null));
        assertEquals("yesterday home comedy", PageMemory.page("yesterday", "home", "comedy"));
    }

    /**
     * A profile that has not been chosen yet renders as the four letters a Kotlin template renders
     * it as.
     *
     * Not a tidy-up anybody should make. {@code seen.active} is nullable and the Kotlin writes it
     * into the key through a string template, which puts the word null there; a Java port that wrote
     * an empty string instead would build a different key for the same page, so a run that opened
     * Home before a profile was chosen and came back to it after would be two pages.
     */
    @Test
    public void a_page_before_a_profile_is_chosen_keys_the_way_the_kotlin_keys_it() {
        assertEquals("null home ", PageMemory.page(null, "home", null));
    }

    /** A genre is part of the page, so two genres are two places. */
    @Test
    public void two_genres_are_two_pages() {
        assertNotEquals(PageMemory.page("yesterday", "home", "comedy"),
                PageMemory.page("yesterday", "home", "drama"));
        assertNotEquals(PageMemory.page("yesterday", "home", null),
                PageMemory.page("yesterday", "home", "comedy"));
    }

    /** And so is the profile, because two viewers do not share a place. */
    @Test
    public void two_profiles_are_two_pages() {
        assertNotEquals(PageMemory.page("yesterday", "home", null),
                PageMemory.page("tomorrow", "home", null));
    }

    /** A page the viewer has not been on holds nothing. */
    @Test
    public void a_page_the_viewer_has_not_been_on_holds_nothing() {
        assertNull(new PageMemory().get(PageMemory.page("yesterday", "home", null)));
        assertEquals(0, new PageMemory().size());
    }

    /**
     * A page's state is built once and found again, which is what keeps the viewer's place for the
     * run of the app.
     */
    @Test
    public void a_page_is_built_once_and_found_again() {
        PageMemory memory = new PageMemory();
        String page = PageMemory.page("yesterday", "home", null);
        Object first = memory.getOrPut(page, RowMemory::new);
        Object again = memory.getOrPut(page, RowMemory::new);
        assertSame("the second visit finds the first visit's state", first, again);
        assertEquals(1, memory.size());
        Object other = memory.getOrPut(PageMemory.page("yesterday", "home", "comedy"), RowMemory::new);
        assertNotEquals(first, other);
        assertEquals(2, memory.size());
    }

    private static final String BROWSE =
            "apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Browse.kt";
}
