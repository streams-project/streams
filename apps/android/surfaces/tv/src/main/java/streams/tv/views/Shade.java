package streams.tv.views;

import android.graphics.Canvas;
import android.graphics.ColorFilter;
import android.graphics.LinearGradient;
import android.graphics.Paint;
import android.graphics.PixelFormat;
import android.graphics.Rect;
import android.graphics.Shader;
import android.graphics.drawable.Drawable;

/**
 * A shade across a part, with its stops where they were measured.
 *
 * <h2>Why this exists rather than a {@link android.graphics.drawable.GradientDrawable}</h2>
 * {@code GradientDrawable} spaces its colours evenly and has no way to say where a stop is. The
 * shades on this surface are not evenly spaced and the positions are measurements:
 * {@code tv/browse/HeroCard.kt:43} fades from black at 0.8 strength to nothing by <b>0.62</b> across
 * the card, and {@code :44} holds nothing until <b>0.55</b> down it and then darkens to 0.55
 * strength. Those two numbers are the shape of the shade. Spacing three colours evenly instead would
 * put the first one at 0.5 and the second at 1.0, which is a different picture and loses both
 * measurements from the source.
 *
 * So this wraps {@link LinearGradient}, which takes the positions, in the smallest drawable that can
 * carry it: one {@code onDraw}, one {@code Paint}, and the shader rebuilt when the bounds change.
 * {@code Brush.horizontalGradient} and {@code Brush.verticalGradient} are the same primitive on the
 * Compose side, so the drawn result is the same gradient and not an approximation of it.
 */
public final class Shade extends Drawable {
    private final int[] colors;
    private final float[] stops;
    private final boolean down;
    private final boolean fromEnd;
    private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);

    private Shade(int[] colors, float[] stops, boolean down, boolean fromEnd) {
        this.colors = colors;
        this.stops = stops;
        this.down = down;
        this.fromEnd = fromEnd;
    }

    /** A shade down a part: {@code Brush.verticalGradient}. */
    public static Shade down(int[] colors, float[] stops) {
        return new Shade(colors, stops, true, false);
    }

    /**
     * A shade across a part, starting on the side the words start from.
     *
     * The twin of {@code fromStart} at {@code tv/app/Round.kt:66-71}, which is one of the two places
     * Compose read the layout direction. Here the caller passes the one mirror switch the layout
     * already has ({@link RefLayout#isMirrored()}), so there is no second reading of a direction and
     * no composition local to provide.
     */
    public static Shade across(int[] colors, float[] stops, boolean mirrored) {
        return new Shade(colors, stops, false, mirrored);
    }

    @Override
    protected void onBoundsChange(Rect bounds) {
        super.onBoundsChange(bounds);
        if (bounds.width() <= 0 || bounds.height() <= 0) {
            paint.setShader(null);
            return;
        }
        float x0 = down ? 0 : (fromEnd ? bounds.right : bounds.left);
        float x1 = down ? 0 : (fromEnd ? bounds.left : bounds.right);
        float y0 = down ? bounds.top : 0;
        float y1 = down ? bounds.bottom : 0;
        paint.setShader(new LinearGradient(x0, y0, x1, y1, colors, stops, Shader.TileMode.CLAMP));
    }

    @Override
    public void draw(Canvas canvas) {
        if (paint.getShader() != null) {
            canvas.drawRect(getBounds(), paint);
        }
    }

    @Override
    public void setAlpha(int alpha) {
        paint.setAlpha(alpha);
        invalidateSelf();
    }

    @Override
    public void setColorFilter(ColorFilter filter) {
        paint.setColorFilter(filter);
        invalidateSelf();
    }

    @Override
    public int getOpacity() {
        return PixelFormat.TRANSLUCENT;
    }
}
