package streams.tv.views;

/**
 * The brand's colours, as the ARGB integers Views draws with.
 *
 * <h2>Why this file is a second home, and what makes that safe</h2>
 * {@code shared/Theme.kt:16-21} holds the same six colours as Compose {@code Color} values, and a
 * Compose {@code Color} is a {@code @JvmInline value class} over a {@code ULong}: Java source cannot
 * name its type and cannot read {@code Back}. So a Views surface has to state them, and stating a
 * brand colour twice is exactly the kind of drift the repository's one-copy rule exists to stop.
 *
 * What makes it safe is that the two copies are held equal by a test rather than by care:
 * {@code src/test/java/streams/tv/browse/NumbersTest.java} reads the six literals out of
 * {@code Theme.kt} and asserts each one against the constant here. A colour changed on one side and
 * not the other fails before a screen is ever drawn.
 *
 * <h2>Why not read them from {@code brand.json} like the corners</h2>
 * The corners already come through {@code BuildConfig} ({@code shared/build.gradle.kts:14-19}),
 * which is generated Java and callable. The colours do not: {@code Theme.kt}'s own comment says
 * "brand.json is the source, and {@code streams build android} will write these", and today nothing
 * writes them. Adding a second generator for them is a build change, and
 * {@code apps/android}'s build files belong to wave 0's package S4 and not to this one. So
 * this file carries them with a test, and the comment says where they should end up.
 */
public final class Ink {
    /** The page behind everything. {@code Theme.kt:16}. */
    public static final int BACK = 0xFF0B0B10;

    /** A panel lifted off the page: the hero. {@code Theme.kt:17}. */
    public static final int PLATE = 0xFF16161D;

    /** A tile before its artwork arrives. {@code Theme.kt:18}. */
    public static final int RAISED = 0xFF24242E;

    /** Words that are not the point: a fact, a dot between facts. {@code Theme.kt:19}. */
    public static final int MUTED = 0xFFA6A6B3;

    /** A profile's plate. {@code Theme.kt:20}. */
    public static final int ACCENT = 0xFF2F6BFF;

    /** A kid's profile plate. {@code Theme.kt:21}. */
    public static final int JOIN = 0xFFE5352B;

    private Ink() {
    }

    /**
     * White at a share of its strength, which is what {@code Color.White.copy(alpha = f)} is at the
     * eight call sites of this folder.
     *
     * The share is given as hundredths and not as a float, for the reason note 210 section 6 gives
     * for every other number here: {@code white(28)} is the 0.28 written at
     * {@code tv/browse/HeroCard.kt:57}, and a reader finds the measurement on the line.
     *
     * <h2>Plain arithmetic and not {@code Color.argb}, which a test found</h2>
     * {@code android.graphics.Color} is one of the classes {@code tv/build.gradle.kts:21}'s
     * {@code unitTests.isReturnDefaultValues = true} stubs, so every call to {@code Color.argb}
     * answers 0 on this machine's JVM and a test of these three could not tell a right answer from a
     * wrong one -- it read 0 for every share. These are integer arithmetic over one number, they
     * need no platform at all, and written this way {@code NumbersTest} can hold them to the shares
     * the Kotlin asks for. The same applies to any other helper here that is only arithmetic.
     */
    public static int white(int hundredths) {
        return shade(0xFFFFFF, hundredths);
    }

    /** Black at a share of its strength: the two shades over the hero, the plate behind a badge. */
    public static int black(int hundredths) {
        return shade(0x000000, hundredths);
    }

    /** A colour at a share of the strength it already has, for a whole row or card drawn dim. */
    public static int dim(int color, int hundredths) {
        int alpha = Math.round(((color >>> 24) & 0xFF) * hundredths / 100f);
        return (Math.max(0, Math.min(255, alpha)) << 24) | (color & 0xFFFFFF);
    }

    /** One colour at one share, rounded as Compose rounds an alpha: to the nearest 255th. */
    private static int shade(int rgb, int hundredths) {
        int alpha = Math.round(255 * hundredths / 100f);
        return (Math.max(0, Math.min(255, alpha)) << 24) | (rgb & 0xFFFFFF);
    }
}
