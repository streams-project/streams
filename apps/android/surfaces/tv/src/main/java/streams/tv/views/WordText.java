package streams.tv.views;

import android.content.Context;
import android.util.TypedValue;
import android.widget.TextView;

/**
 * A line of the interface's words: it holds the key and reads it, rather than holding the words.
 *
 * Why the key and not the text: the words arrive from the server and change inside a run when the
 * viewer changes language ({@code Shell.speak}), so a part that was handed a string would keep the
 * old language until whoever built it remembered to rebuild it. A part that holds its key is told to
 * read again -- {@link Words#into} walks the screen once -- and the words it draws cannot be stale.
 *
 * <h2>The empty rule, which is not a fallback</h2>
 * A key the words do not hold draws nothing. Not the key, not English, not a dash: nothing, and no
 * content description either. {@code Says.kt:46-52} and {@code Spoken.kt:41-42} give the reason --
 * the words are empty for as long as a run has not read its language, and a part labelled with a
 * placeholder in that window is read out by a screen reader as a placeholder. A part that draws
 * nothing is a visible gap and gets fixed; a part that draws its key looks like a translation
 * mistake and ships.
 *
 * <h2>Text size</h2>
 * {@code setTextSize(COMPLEX_UNIT_SP, Ref.sp(44))} is the Views form of {@code 44.refType}
 * ({@code shared/Measure.kt:15}), and 44 is the number written at {@code tv/browse/Rows.kt:104}. In
 * sp and not pixels because a language may ask for larger text ({@code tv/app/Television.kt:56}).
 */
public class WordText extends TextView implements Words.Bound {
    private String key = "";
    private String base = "";
    private int count;

    public WordText(Context context) {
        super(context);
    }

    /** A line of words at a measured size. */
    public static WordText of(Context context, String key, int refType) {
        WordText text = new WordText(context);
        text.setKey(key);
        text.size(refType);
        return text;
    }

    /** The key this line reads. */
    public final void setKey(String key) {
        this.key = key == null ? "" : key;
        this.base = "";
    }

    public final String key() {
        return key;
    }

    /**
     * A counted line: the key family and the number, so the line reads the right plural form of the
     * language it is in when it reads again.
     */
    public final void setCount(String base, int count) {
        this.base = base == null ? "" : base;
        this.count = count;
        this.key = "";
    }

    /** This line's size as it was measured off the capture, in reference pixels. */
    public final void size(int refType) {
        setTextSize(TypedValue.COMPLEX_UNIT_SP, Ref.sp(refType));
    }

    /**
     * Reads this line's key. What is drawn is what the words hold and nothing else.
     *
     * No content description is set: a line of text is its own label in Views, so setting one would
     * be a second copy of the same words that a screen reader reads instead of what is drawn.
     */
    @Override
    public void words(Words words) {
        setText(base.isEmpty() ? words.of(key) : words.counted(base, count));
    }
}
