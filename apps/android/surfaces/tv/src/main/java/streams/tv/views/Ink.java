package streams.tv.views;

import streams.shared.Colour;

/**
 * The brand's colours, as the ARGB integers Views draws with.
 *
 * <h2>Why the colours are read and not restated</h2>
 * {@code Colour.BACKGROUND} and its five siblings are {@code const val} on a Kotlin object
 * ({@code shared/Theme.kt:155-162}) reading {@code BuildConfig} integers that
 * {@code shared/build.gradle.kts:30-40} writes from {@code brand.json}. A {@code const val} is a
 * static field on the class, so Java calls it, and generated {@code BuildConfig} is Java already.
 * That is the path {@link Round} takes for the corners, and it now carries the colours too: the
 * brand's colour has one home for both languages. This class only names the roles a television
 * screen draws and adds the shades.
 *
 * <h2>What was here until 2026-11-12, and why it is gone</h2>
 * A Compose {@code Color} is a {@code @JvmInline value class} over a {@code ULong}, so Java source
 * cannot name its type and cannot read {@code Back}. While {@code Theme.kt} held the six as Compose
 * literals, this file had to restate all six as ARGB ints, and the two copies were held equal by a
 * test rather than by care -- a brand colour with two homes, which is the drift the repository's
 * one-copy rule exists to stop. An ARGB int is a form both toolkits take ({@code Color(Int)} on one
 * side, {@code GradientDrawable.setColor} on the other), so the generator emits that and each
 * language wraps it. {@code NumbersTest.java} no longer holds two copies together: it holds each
 * constant below against the colour {@code brand.json} gives for that role.
 */
public final class Ink {
    /** The page behind everything. {@code brand.json} {@code background}. */
    public static final int BACK = Colour.BACKGROUND;

    /** A panel lifted off the page: the hero. {@code brand.json} {@code surface}. */
    public static final int PLATE = Colour.SURFACE;

    /** A tile before its artwork arrives. {@code brand.json} {@code raised}. */
    public static final int RAISED = Colour.RAISED;

    /** Words that are not the point: a fact, a dot between facts. {@code brand.json} {@code muted}. */
    public static final int MUTED = Colour.MUTED;

    /** A profile's plate. {@code brand.json} {@code accent}. */
    public static final int ACCENT = Colour.ACCENT;

    /** A kid's profile plate. {@code brand.json} {@code join}. */
    public static final int JOIN = Colour.JOIN;

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
