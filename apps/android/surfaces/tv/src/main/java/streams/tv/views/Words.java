package streams.tv.views;

import android.view.View;
import android.view.ViewGroup;
import java.util.Collections;
import java.util.Map;
import streams.shared.Speaking;
import streams.shared.ThemeKt;

/**
 * The words of the interface, bound to views at run time.
 *
 * <h2>Why {@code strings.xml} is not an option, rather than not preferred</h2>
 * The words do not exist when the app is built. They arrive from the server, per language, as
 * {@code Seen.words: Map<String, String>} ({@code apps/android/ports/.../Ports.kt:184}), published
 * through {@code Shell.seen} ({@code :214}) and refilled by {@code Shell.speak(language)}
 * ({@code :238}) when the viewer changes language inside the app. There is no
 * {@code res/values/strings.xml} in this tree and the only thing that ever writes a string resource
 * is {@code app/build.gradle.kts:30}, which writes the app's own name from {@code brand.json}.
 * So an Android resource could hold a build-time copy of one language, which is a second home for
 * every word and a way for English to appear on a Japanese television.
 *
 * <h2>The three entry points</h2>
 * {@link #of(String)} for code that needs a word; {@link #counted(String, int)} for a word with a
 * number in it; {@link #into(View)} to rebind a whole screen when the language changes.
 *
 * <h2>The empty answer, which has to stay exactly as it is</h2>
 * A key the words do not hold answers empty, and empty is never replaced with English, with the key
 * itself, or with anything else. The reason is at {@code shared/Says.kt:46-52} and
 * {@code tv/app/Spoken.kt:41-42}: {@code seen.words} is empty for as long as a run has not read its
 * language, and a part labelled with a placeholder in that window is worse than one a reader falls
 * through. {@link WordText} is where that rule is applied to a view.
 *
 * <h2>Why the plural rules are not copied in here</h2>
 * {@link #counted} is {@code shared/Theme.kt:131}'s shape, but the CLDR plural table
 * ({@code Theme.kt:100-128}), the {@code {name}} filling with its bidirectional isolates
 * ({@code :66-67}) and the Arabic-Indic digits ({@code :86-96}) are called, not re-written. They are
 * one statement of one rule that two languages now read, which is the thing
 * {@code platform/tools/check_patterns.py} exists to protect; a second copy in Java would be a
 * second place for a language's plural forms to be wrong. When the Kotlin goes, that table moves
 * once -- it does not get translated twice.
 */
public final class Words {
    private Map<String, String> book = Collections.emptyMap();

    /** A view that holds a key and can be told to read it again. */
    public interface Bound {
        void words(Words words);
    }

    /**
     * The words a fresh {@code Seen} carries. True when they are not the ones that were held, which
     * is when a screen has to rebind.
     *
     * The same map is the same words, by identity and not by content: a {@code Seen} that did not
     * change its language carries the map it carried before, and comparing three hundred entries on
     * every arrival to learn that would cost more than the rebind it saves.
     */
    public boolean read(Map<String, String> words) {
        Map<String, String> fresh = words == null ? Collections.<String, String>emptyMap() : words;
        if (fresh == book) {
            return false;
        }
        book = fresh;
        return true;
    }

    /** How many words are held. Zero until a run has read its language. */
    public int size() {
        return book.size();
    }

    /**
     * The word for a key, or empty. Never English, never the key.
     */
    public String of(String key) {
        String said = book.get(key);
        return said == null ? "" : said;
    }

    /**
     * A counted word: {@code {base}.{form}} for the count's plural form in the interface's
     * language, falling back to {@code other}, then {@code many}, then {@code one}, with
     * {@code {n}} filled in.
     *
     * The fallback order is {@code Theme.kt:131-134}'s and is not arbitrary. English holds
     * {@code title.season.one} and {@code title.season.many} and no {@code other}, so two seasons
     * asks for a form English does not have and has to land on {@code many} and not on {@code one}.
     * That is the case the test pins.
     */
    public String counted(String base, int n) {
        String form = ThemeKt.pluralOf(n, Speaking.INSTANCE.getCode());
        String said = firstOf(base + "." + form, base + ".other", base + ".many", base + ".one");
        return said.isEmpty() ? "" : ThemeKt.fill(said, "n", n);
    }

    private String firstOf(String... keys) {
        for (String key : keys) {
            String said = book.get(key);
            if (said != null) {
                return said;
            }
        }
        return "";
    }

    /**
     * Rebinds every word in a screen: walked once when the screen is built, and again when
     * {@code seen.language} changes.
     *
     * The walk is over views that say they hold a key ({@link Bound}), so a screen declares its
     * keys where it builds its parts and nothing has to keep a list of them. Note 210's F1 reads
     * the key from an {@code AttributeSet} instead, through a declared {@code wordKey} attribute on
     * a styleable; that form belongs to a surface built from layout files, and this one is not --
     * note 214 measured XML inflation at 22.0 ms against {@code setContent}'s 9.0 on a Streamer, so
     * note 210's own wave 3 builds the television's views in code. With no layout file there is no
     * {@code AttributeSet} to read, and a key set where the view is built is checked by the compiler
     * rather than by a string in XML.
     */
    public void into(View root) {
        if (root instanceof Bound) {
            ((Bound) root).words(this);
        }
        if (root instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) root;
            for (int index = 0; index < group.getChildCount(); index++) {
                into(group.getChildAt(index));
            }
        }
    }
}
