package streams.tv.browse;

import android.content.Context;
import android.graphics.Bitmap;
import android.widget.ImageView;
import streams.tv.views.Ref;
import streams.tv.views.Says;

/**
 * A picture filling its box, read through the {@code Pictures} port.
 *
 * The Views twin of {@code shared/Art.kt:36-53}, which is one {@code AsyncImage}. Three things are
 * worth saying about it, because this one small class settles three open items at once.
 *
 * <h2>1. It is the first consumer the {@code Pictures} port has ever had</h2>
 * {@code Shell.pictures} exists, the adapter behind it exists
 * ({@code adapters/.../CoilPictures.kt}), and note 210 measured that <b>zero</b> surface files call
 * it: every picture in the app goes through {@code shared/Art.kt:52}'s {@code AsyncImage}, which is
 * why {@code check_rings.py:117}'s exemption for {@code io.coil-kt.coil3:coil-compose} is still
 * load bearing. Note 210's risk table also records that the television surface has no disk cache at
 * all today -- {@code shared/Loader.kt:21-23} says the television never installs the process loader,
 * so it runs on Coil's default -- and that wave 1's F2 was meant to fix it by routing the television
 * through the port, whose adapter owns the 64 MB cache. F2 was not built. This is it, for this page.
 *
 * <h2>2. The box is asked for, not left to the layout</h2>
 * {@code Want.wide} and {@code Want.high} are given in pixels, which is what the port's own comment
 * asks for: the read starts before the first measure instead of a frame after it, and a picture read
 * ahead of the viewer lands under the same key as the one that is drawn. Every part of this page
 * knows its size as a measured number, so the box is known before the view exists -- which is the
 * one thing this surface's fixed reference geometry makes easy and a flowing layout does not.
 *
 * <h2>3. Nothing fades in</h2>
 * {@code shared/Art.kt:29-30}: the reference paints a flat tile and then the artwork, with no
 * shimmer and no crossfade. So the plate stays behind the picture and the picture is set when it
 * arrives. The one crossfade on this page is the open card's own ({@code tv/browse/Rows.kt:93}),
 * which is a different thing: the same card changing which title it shows.
 *
 * <h2>Why a {@link Source} and not {@code Pictures} itself</h2>
 * {@code Pictures.fetch} takes {@code (Picture) -> Unit}, a Kotlin function type, and {@code Want}
 * has four parameters with defaults. Both are callable from Java only as
 * {@code kotlin.jvm.functions.Function1} and a four argument constructor. Note 210's wave 0 package
 * S2 exists to turn exactly those into Java callable single method interfaces, and it owns
 * {@code apps/android/ports/**}, which this package does not. So the interface is declared here, in
 * the shape S2 will give the port, and the one line that adapts it lives in {@code browse/Reading.kt}
 * beside the other Kotlin that this gate makes necessary. When S2 lands, this interface is deleted
 * and {@code Pictures} is passed straight in.
 */
public final class Art extends ImageView {
    /** Where a picture comes from, in a shape Java can call. */
    public interface Source {
        /**
         * Reads one picture at a size, and answers once. The returned {@link Stop} abandons the
         * read, which matters on a row the viewer has moved off.
         *
         * @param wide the box's width in pixels, or 0 for the picture's own size
         */
        Stop fetch(String url, int wide, int high, Done done);
    }

    /** The right to stop one read. {@code ports.Stop}. */
    public interface Stop {
        void stop();
    }

    /** A picture, or nothing. Null means the read did not produce one and something else is drawn. */
    public interface Done {
        void painted(Bitmap picture);
    }

    private final Source source;
    private String url = "";
    private Stop reading;

    /**
     * @param says the words a screen reader is given, or null for a picture that repeats what is
     *             already said beside it. Null is the right answer for most of them, and
     *             {@code shared/Art.kt:31-34} gives the reason: the art inside a tile is the tile,
     *             and the tile already says the title's name, so the picture is decoration and
     *             marking it so keeps a reader from stopping twice on one thing. Marking it is not
     *             optional -- an unmarked decoration is one more node a reader reads out as nothing,
     *             which is the defect {@code Says.kt:16-21} measured.
     */
    public Art(Context context, Source source, String says) {
        super(context);
        this.source = source;
        setScaleType(ScaleType.CENTER_CROP);
        Says.onto(this, says == null ? Says.decoration() : Says.says(says));
    }

    /**
     * Draws this picture, at this reference size. The same url twice is not read twice, which is
     * what keeps a redraw from re-reading every poster on the page.
     */
    public void show(String url, int refWide, int refHigh) {
        String wanted = url == null ? "" : url;
        if (wanted.equals(this.url)) {
            return;
        }
        this.url = wanted;
        abandon();
        setImageBitmap(null);
        if (wanted.isEmpty() || source == null) {
            return;
        }
        final String asked = wanted;
        reading = source.fetch(wanted, Ref.px(getContext(), refWide), Ref.px(getContext(), refHigh),
                new Done() {
                    @Override
                    public void painted(Bitmap picture) {
                        // The read answers on the thread that asked, which is this one, but a row
                        // the viewer moved off may have asked for something else since. So the
                        // answer is dropped unless it is still the one wanted.
                        if (picture != null && asked.equals(Art.this.url)) {
                            setImageBitmap(picture);
                        }
                    }
                });
    }

    /** Stops the read, for a part going off screen or being pointed at something else. */
    public void abandon() {
        if (reading != null) {
            reading.stop();
            reading = null;
        }
    }

    @Override
    protected void onDetachedFromWindow() {
        abandon();
        super.onDetachedFromWindow();
    }
}
