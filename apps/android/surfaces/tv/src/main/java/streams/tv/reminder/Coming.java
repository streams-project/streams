package streams.tv.reminder;

import contract.Contract;
import java.util.List;

/**
 * One title that has not arrived yet, and whether this viewer asked to be told when it does.
 *
 * <p>Read straight out of {@code GET /v1/titles/{key}}'s {@code TitleDetail} by {@link
 * #read(String)}. Java can name the contract since note 240, so this screen has no Kotlin seam
 * either; what it still asks a host for is the shell, because {@code Shell.act} takes
 * {@code contract.Action} -- the <em>Kotlin</em> arm of the generated types, a different class from
 * {@code contract.Contract.Action} and with no cast between them. That one call is the seam, and it
 * is {@code ReminderView.Asked}.
 *
 * <h2>Why a reminder is a screen here and a button on the phone</h2>
 * The phone puts Remind Me on the title page ({@code phone/title/Title.kt:312-349}). The television
 * cannot: the title page is Compose and belongs to another hand this week, and a remote has no
 * thumb to reach a button with. A screen of its own is also the television's own grammar for a
 * question with two answers -- {@code settings/Settings.kt:245-248} is the same two line panel --
 * and it is reachable by an address, so the title page gains it with one line when it is ready.
 */
public final class Coming {
    /** Nothing known yet: what the screen draws before its answer. */
    public static final Coming NOTHING = new Coming("", "", "", "", false);

    /** The title's key as a plain string, for the host to turn back into an address or a command. */
    public final String key;

    /** The title's own name, from the catalogue. */
    public final String name;

    /** The poster, or empty. */
    public final String poster;

    /** The day it arrives, as the server sent it. Empty when the title is already here. */
    public final String day;

    /** Whether this profile has already asked to be told. */
    public final boolean set;

    public Coming(String key, String name, String poster, String day, boolean set) {
        this.key = key == null ? "" : key;
        this.name = name == null ? "" : name;
        this.poster = poster == null ? "" : poster;
        this.day = day == null ? "" : day;
        this.set = set;
    }

    /**
     * True when there is something to ask about: a title with a name that has not arrived.
     *
     * A title already available has no {@code available_from} and nothing to be reminded of, so the
     * screen says it knows nothing rather than offering a reminder for a thing a viewer can watch
     * now. That is the same condition the phone's button hangs on
     * ({@code Title.kt:300}, {@code if (availableFrom != null)}).
     */
    public boolean known() {
        return !name.isEmpty() && !day.isEmpty();
    }

    /** The same title with the reminder the other way, for drawing the answer before the server has it. */
    public Coming turned(boolean on) {
        return on == set ? this : new Coming(key, name, poster, day, on);
    }

    /**
     * One answer to {@code GET /v1/titles/{key}}, read.
     *
     * A body this build cannot read answers {@link #NOTHING}, for the reason {@code Notices.read}
     * gives: this screen is already the one that says it knows nothing, and a parse failure that
     * threw would leave a television with a dead screen and no way back.
     */
    public static Coming read(String body) {
        if (body == null || body.isEmpty()) {
            return NOTHING;
        }
        Contract.TitleDetail detail;
        try {
            detail = Contract.TitleDetail.read(Contract.Json.parse(body));
        } catch (RuntimeException broken) {
            return NOTHING;
        }
        return new Coming(
                detail.getTitle().value(),
                detail.getName(),
                poster(detail.getImagesList()),
                detail.hasAvailableFrom() ? detail.getAvailableFrom().value() : "",
                detail.hasReminderSet() && detail.getReminderSet());
    }

    /**
     * The picture for a tall place: {@code shared/Art.kt:22}'s {@code tall}, poster first and the
     * wide pictures behind it.
     *
     * Written out rather than called for the reason {@code Notices.still} gives: {@code tall} takes
     * the Kotlin arm's {@code List<contract.Image>} and this is the Java arm's. The order is held
     * against the Kotlin by {@code ComingTest}.
     */
    static String poster(List<Contract.Image> images) {
        String found = roled(images, Contract.ImageRole.POSTER);
        if (found.isEmpty()) {
            found = roled(images, Contract.ImageRole.BACKDROP);
        }
        if (found.isEmpty()) {
            found = roled(images, Contract.ImageRole.STILL);
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
