package streams.tv.notifications;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

import contract.Contract;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.junit.Test;

/**
 * One answer to {@code GET /v1/notifications}, read by Java straight out of the contract.
 *
 * <p>Note 240 opened this on 2026-11-13 and {@code ports/.../ContractFromJavaTest.java} proved the
 * gate; this is the first screen to walk through it. {@code browse/Feed.java} still says Java
 * source cannot read the contract, and its own evidence -- four getters whose names hold a hyphen
 * -- was true of the Kotlin arm and not of the Java one.
 *
 * <p>The bodies below are written as wire text and not as objects, because what is being checked is
 * the reading: a field renamed on the server is a body this screen stops understanding, and a test
 * that built {@code Contract.Notifications} by its constructor would never notice.
 */
public class NoticesTest {
    private static final String TWO = "{"
            + "\"cursor\":\"c2\",\"has_more\":true,\"items\":["
            + "{\"id\":\"n1\",\"unread\":true,\"date\":\"2026-12-01\","
            + "\"kind\":\"coming\","
            + "\"tile\":{\"title\":\"movie:7\",\"name\":\"Dune\",\"images\":["
            + "{\"role\":\"poster\",\"url\":\"http://box/p7.jpg\"},"
            + "{\"role\":\"backdrop\",\"url\":\"http://box/b7.jpg\"}]}},"
            + "{\"id\":\"n2\",\"unread\":false,\"date\":\"2026-11-30\","
            + "\"kind\":\"new_season\","
            + "\"tile\":{\"title\":\"tv:9\",\"name\":\"Stranger Things\"}}"
            + "]}";

    @Test
    public void a_body_from_the_server_becomes_the_lines_the_screen_draws() {
        Notices feed = Notices.read(TWO);
        assertEquals(2, feed.size());
        assertEquals("c2", feed.cursor);
        assertTrue(feed.more);

        Notices.Notice first = feed.at(0);
        assertEquals("n1", first.id);
        assertEquals("movie:7", first.key);
        assertEquals("Dune", first.name);
        assertEquals("2026-12-01", first.day);
        assertTrue(first.unread);
        assertEquals("notification.coming", first.kind);
    }

    @Test
    public void a_tile_with_no_pictures_asks_for_none() {
        assertEquals("", Notices.read(TWO).at(1).still);
    }

    @Test
    public void past_the_end_is_nothing_rather_than_a_crash() {
        Notices feed = Notices.read(TWO);
        assertNull(feed.at(2));
        assertNull(feed.at(-1));
        assertEquals(1, feed.last());
    }

    /**
     * A body this build cannot read is an empty screen and not a dead one.
     *
     * The screen is already the one that says there is nothing here. A parse failure that threw
     * would leave a television showing a blank page with no way back, which is the defect note 239
     * measured on a signed in television whose server had gone: 9 nodes, 0 focusable, 0 words, for
     * as long as anyone looked at it.
     */
    @Test
    public void a_body_this_build_cannot_read_is_an_empty_screen() {
        assertEquals(0, Notices.read("{\"has_more\":").size());
        assertEquals(0, Notices.read("{\"items\":[{\"id\":\"n1\"}],\"has_more\":false}").size());
        assertEquals(0, Notices.read("").size());
        assertEquals(0, Notices.read(null).size());
    }

    // --- the five kinds, and the sixth ----------------------------------------------------------

    /**
     * Every kind the contract has, and its key. Read off {@code Contract.NoticeKind} rather than
     * listed here, so a sixth kind added to the contract fails this test instead of silently
     * arriving on the television with no words.
     */
    @Test
    public void every_kind_the_contract_has_is_given_a_key() {
        List<Contract.NoticeKind> all = Arrays.asList(
                Contract.NoticeKind.NEW_ARRIVAL, Contract.NoticeKind.COMING,
                Contract.NoticeKind.REMINDER_LIVE, Contract.NoticeKind.NEW_SEASON,
                Contract.NoticeKind.NEW_EPISODE);
        List<String> keys = new ArrayList<String>();
        for (int at = 0; at < all.size(); at++) {
            String key = Notices.kindKey(all.get(at));
            assertFalse(all.get(at).value() + " has no key", key.isEmpty());
            keys.add(key);
        }
        assertEquals(Arrays.asList(
                "notification.new_arrival", "notification.coming", "notification.reminder_live",
                "notification.new_season", "notification.new_episode"), keys);
        assertEquals("the contract holds five kinds; a sixth needs a key and a word",
                5, howManyKindsTheContractHas());
    }

    /** Counted off the generated Java, not written down. */
    private static int howManyKindsTheContractHas() {
        String source = Reading.text("platform/contract/generated/types/Contract.java");
        int at = source.indexOf("public static final class NoticeKind");
        assertTrue("NoticeKind has gone from the contract", at > 0);
        String body = source.substring(at, source.indexOf("private final String value;", at));
        int found = 0;
        int from = 0;
        while (true) {
            int next = body.indexOf("new NoticeKind(\"", from);
            if (next < 0) {
                return found;
            }
            found += 1;
            from = next + 1;
        }
    }

    @Test
    public void a_kind_from_a_newer_server_is_drawn_with_no_words_rather_than_its_wire_value() {
        assertEquals("", Notices.kindKey(new Contract.NoticeKind("flash_sale")));
        assertEquals("", Notices.kindKey(null));
        String body = TWO.replace("\"coming\"", "\"flash_sale\"");
        Notices.Notice odd = Notices.read(body).at(0);
        assertEquals("", odd.kind);
        assertEquals("and the notice still carries what happened", "Dune", odd.name);
    }

    // --- the picture ----------------------------------------------------------------------------

    @Test
    public void the_still_is_the_wide_picture_and_the_poster_is_the_last_resort() {
        assertEquals("http://box/b7.jpg", Notices.read(TWO).at(0).still);
        assertEquals("http://box/p7.jpg",
                Notices.read(TWO.replace("{\"role\":\"backdrop\",\"url\":\"http://box/b7.jpg\"}",
                        "{\"role\":\"logo\",\"url\":\"http://box/l7.jpg\"}")).at(0).still);
    }

    /**
     * And the order is the Kotlin's own, held against the file that states it.
     *
     * {@code Notices.still} is the one rule in this package written twice, because {@code wide}
     * takes the Kotlin arm's {@code List<contract.Image>} and this takes the Java arm's. Two
     * statements of one rule drift, so this reads the Kotlin and asserts the three roles are in the
     * same order there as here.
     */
    @Test
    public void the_order_is_the_one_shared_art_states() {
        String kotlin = Reading.text(
                "apps/android/surfaces/shared/src/main/kotlin/streams/shared/Art.kt");
        int from = kotlin.indexOf("fun wide(");
        assertTrue("shared/Art.kt no longer has wide()", from > 0);
        String line = kotlin.substring(from, kotlin.indexOf("/** The picture for a tall place", from));
        assertTrue("backdrop before still",
                line.indexOf("ImageRole.BACKDROP") < line.indexOf("ImageRole.STILL"));
        assertTrue("still before poster",
                line.indexOf("ImageRole.STILL") < line.indexOf("ImageRole.POSTER"));
    }

    // --- what a page of a feed does with the next one -------------------------------------------

    @Test
    public void the_next_page_joins_on_the_end_and_brings_the_cursor_with_it() {
        Notices first = Notices.read(TWO);
        Notices later = Notices.read(TWO.replace("\"c2\"", "\"c3\"")
                .replace("\"has_more\":true", "\"has_more\":false")
                .replace("\"n1\"", "\"n3\"").replace("\"n2\"", "\"n4\""));
        Notices whole = first.and(later);
        assertEquals(4, whole.size());
        assertEquals("the later answer describes what is left", "c3", whole.cursor);
        assertFalse(whole.more);
        assertEquals("n3", whole.at(2).id);
    }

    @Test
    public void reading_a_notice_reads_everything_newer_than_it() {
        Notices feed = Notices.read(TWO.replace("\"unread\":false", "\"unread\":true"));
        assertEquals(2, feed.unread());
        Notices after = feed.readTo("n1");
        assertFalse(after.at(0).unread);
        assertTrue("and leaves the older ones alone", after.at(1).unread);
        assertEquals(1, after.unread());
        assertEquals("a notice that is not here changes nothing", 1, after.readTo("n9").unread());
    }

    @Test
    public void a_notice_is_found_by_its_id() {
        Notices feed = Notices.read(TWO);
        assertEquals(1, feed.placeOf("n2"));
        assertEquals(-1, feed.placeOf("n9"));
        assertEquals(-1, feed.placeOf(""));
        assertEquals(-1, feed.placeOf(null));
    }
}
