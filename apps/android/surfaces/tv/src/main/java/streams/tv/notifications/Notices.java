package streams.tv.notifications;

import contract.Contract;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * What the notifications screen draws: one page of the feed, flattened out of the contract.
 *
 * <h2>Why this reads the contract itself, where {@code browse/Feed.java} could not</h2>
 * {@code Feed.java} says Java source cannot read the contract and names the four mangled getters
 * that prove it, so one Kotlin file fills it. That sentence was true on 2026-11-12 and is not true
 * now: note 240 added {@code platform/contract/generated/types} to the <em>java</em> source set of
 * {@code :ports} on 2026-11-13, and {@code contract.Contract$Notifications} is a plain Java class
 * with a public constructor and unmangled getters. So this screen has no Kotlin seam at all, and
 * {@link #read(String)} is the whole of its reading.
 *
 * <p>The two arms are different classes and not two views of one. {@code contract.Notifications} is
 * the Kotlin arm; {@code contract.Contract.Notifications} is this one. Nothing here can be handed
 * to a function that takes the other, which is why the image choice below is written out rather
 * than calling {@code shared/Art.kt}'s {@code wide}.
 *
 * <h2>What it is not</h2>
 * Not a cache: one arrival, built fresh from one answer. What survives an arrival is where the
 * viewer was, and that lives in {@link Spot}, kept under the notice's own id so a feed that gained
 * a notice at the top does not move the focus down.
 *
 * <h2>And it carries no words</h2>
 * Every string here is either from the server -- the title's name, the day -- or a <em>key</em> into
 * the words of the interface ({@link Notice#kind}). No English is written in this file, which is
 * {@code Words.java}'s rule.
 */
public final class Notices {
    /** An empty feed: what the screen draws before its first answer. */
    public static final Notices NOTHING =
            new Notices(Collections.<Notice>emptyList(), "", false);

    /** One line of the feed. */
    public static final class Notice {
        /** The notice's own id, which is what {@code POST /v1/notifications/read} is told. */
        public final String id;

        /** The title's key as a plain string, for the seam to turn back into an address. */
        public final String key;

        /** The title's own name, from the catalogue, already in the viewer's language. */
        public final String name;

        /** The picture for this line's still, or empty. */
        public final String still;

        /**
         * The <em>key</em> for what kind of notice this is, never the words. Empty for a kind this
         * build does not know, and an empty key reads as no words at all rather than as the kind's
         * wire value, which is {@code Words.of}'s rule: a viewer must never be shown
         * {@code new_season}.
         */
        public final String kind;

        /** The day the server put on it, as it sent it. */
        public final String day;

        public final boolean unread;

        public Notice(String id, String key, String name, String still, String kind, String day,
                boolean unread) {
            this.id = id == null ? "" : id;
            this.key = key == null ? "" : key;
            this.name = name == null ? "" : name;
            this.still = still == null ? "" : still;
            this.kind = kind == null ? "" : kind;
            this.day = day == null ? "" : day;
            this.unread = unread;
        }

        /** The same line, read. Drawn muted and no longer said as unread. */
        public Notice read() {
            return unread ? new Notice(id, key, name, still, kind, day, false) : this;
        }
    }

    public final List<Notice> items;

    /** The cursor for the next page, or empty when there is none. */
    public final String cursor;

    /** Whether the server says there is more behind that cursor. */
    public final boolean more;

    public Notices(List<Notice> items, String cursor, boolean more) {
        this.items = items == null ? Collections.<Notice>emptyList() : items;
        this.cursor = cursor == null ? "" : cursor;
        this.more = more;
    }

    public int size() {
        return items.size();
    }

    /** The last line's place, or -1 for an empty feed. */
    public int last() {
        return items.size() - 1;
    }

    /** The line at a place, or null past the end. */
    public Notice at(int index) {
        return index >= 0 && index < items.size() ? items.get(index) : null;
    }

    /** Where a notice sits now, or -1 when it is not in this feed. */
    public int placeOf(String id) {
        if (id == null || id.isEmpty()) {
            return -1;
        }
        for (int at = 0; at < items.size(); at++) {
            if (items.get(at).id.equals(id)) {
                return at;
            }
        }
        return -1;
    }

    /**
     * The next page joined on the end. The cursor and the flag come from the later answer, because
     * they describe what is left rather than what has arrived.
     */
    public Notices and(Notices later) {
        if (later == null) {
            return this;
        }
        List<Notice> all = new ArrayList<Notice>(items);
        all.addAll(later.items);
        return new Notices(all, later.cursor, later.more);
    }

    /** The same feed with everything down to and including a notice marked read. */
    public Notices readTo(String id) {
        int to = placeOf(id);
        if (to < 0) {
            return this;
        }
        List<Notice> all = new ArrayList<Notice>(items.size());
        for (int at = 0; at < items.size(); at++) {
            all.add(at <= to ? items.get(at).read() : items.get(at));
        }
        return new Notices(all, cursor, more);
    }

    /** How many are unread, for the count a reader is given on the way in. */
    public int unread() {
        int count = 0;
        for (int at = 0; at < items.size(); at++) {
            if (items.get(at).unread) {
                count += 1;
            }
        }
        return count;
    }

    /**
     * One answer to {@code GET /v1/notifications}, read.
     *
     * A body this build cannot read answers {@link #NOTHING} rather than throwing: the screen is
     * already the one that says there is nothing, and a parse failure that killed the screen would
     * turn a server that sent one bad field into a television with no way back.
     */
    public static Notices read(String body) {
        if (body == null || body.isEmpty()) {
            return NOTHING;
        }
        Contract.Notifications answer;
        try {
            answer = Contract.Notifications.read(Contract.Json.parse(body));
        } catch (RuntimeException broken) {
            return NOTHING;
        }
        List<Contract.NoticeItem> sent = answer.getItemsList();
        List<Notice> lines = new ArrayList<Notice>(sent.size());
        for (int at = 0; at < sent.size(); at++) {
            Contract.NoticeItem item = sent.get(at);
            Contract.Tile tile = item.getTile();
            lines.add(new Notice(
                    item.getId(),
                    tile.getTitle().value(),
                    tile.getName(),
                    still(tile.getImagesList()),
                    kindKey(item.getKind()),
                    item.getDate().value(),
                    item.getUnread()));
        }
        return new Notices(lines, answer.getCursor(), answer.getHasMore());
    }

    /**
     * The words key for a kind, or empty for one this build does not know.
     *
     * The five are {@code contract.Contract.NoticeKind}'s own five, and the keys are the five the
     * phone reads at {@code phone/notifications/Notifications.kt:228-234}. A sixth kind arriving
     * from a newer server falls to empty here and draws no chip, which is what the phone's
     * {@code else -> ""} does; the notice itself still draws its name and its day, so a viewer sees
     * the thing that happened even when this build has no word for what kind it was.
     */
    static String kindKey(Contract.NoticeKind kind) {
        if (kind == null) {
            return "";
        }
        if (Contract.NoticeKind.COMING.equals(kind)) {
            return "notification.coming";
        }
        if (Contract.NoticeKind.REMINDER_LIVE.equals(kind)) {
            return "notification.reminder_live";
        }
        if (Contract.NoticeKind.NEW_ARRIVAL.equals(kind)) {
            return "notification.new_arrival";
        }
        if (Contract.NoticeKind.NEW_SEASON.equals(kind)) {
            return "notification.new_season";
        }
        if (Contract.NoticeKind.NEW_EPISODE.equals(kind)) {
            return "notification.new_episode";
        }
        return "";
    }

    /**
     * The picture for this line's still: {@code shared/Art.kt:18-19}'s {@code wide} order, backdrop
     * then still then poster.
     *
     * Written out rather than called, and this is the one place in these two screens where a rule
     * is stated twice. {@code wide} takes {@code List<contract.Image>} -- the Kotlin arm of the
     * generated types -- and what arrives here is {@code List<contract.Contract.Image>}, the Java
     * arm. Note 240 measured them as two classes with different binary names in one jar, so neither
     * can be passed where the other is wanted and there is no cast between them. The order is three
     * lines and is held against the Kotlin by {@code NoticesTest}, so a change to one that is not
     * made to the other fails here rather than drawing the wrong picture on one surface.
     */
    static String still(List<Contract.Image> images) {
        String found = roled(images, Contract.ImageRole.BACKDROP);
        if (found.isEmpty()) {
            found = roled(images, Contract.ImageRole.STILL);
        }
        if (found.isEmpty()) {
            found = roled(images, Contract.ImageRole.POSTER);
        }
        return found;
    }

    private static String roled(List<Contract.Image> images, Contract.ImageRole role) {
        if (images == null) {
            return "";
        }
        for (int at = 0; at < images.size(); at++) {
            Contract.Image image = images.get(at);
            if (role.equals(image.getRole())) {
                return image.getUrl() == null ? "" : image.getUrl();
            }
        }
        return "";
    }
}
