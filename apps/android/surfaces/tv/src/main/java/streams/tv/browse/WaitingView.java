package streams.tv.browse;

import android.content.Context;
import java.util.Map;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Words;

/**
 * What a television page shows while it has nothing to show. {@code tv/browse/Waiting.kt:31-49}.
 *
 * What is drawn is the measured reference behaviour and nothing more, carried from that file:
 * Netflix's television app, between tabs, draws the top bar with the destination tab already lit and
 * the body pure black, for 1.08-2.38 s going Home to Series and 3.96-4.67 s going Films to Games,
 * with no skeleton, no shimmer and no old content held
 * ({@code netflix-reference/00-gaps.md:133}). The touch surface of this project settled the same
 * question for itself by drawing placeholder rows, which is its own reference's behaviour on a phone
 * and not this one's.
 *
 * There is no wording here, invented or otherwise. A page that cannot say where it is -- no tabs
 * yet, which is what a run with no server looks like -- draws the black body alone rather than a bar
 * with nothing in it, which is why the bar's visibility follows the tab count.
 *
 * It is not a {@code RemoteHost}: there is nothing on it the remote can move to, and a focusable
 * node with no state would take the keys and swallow them. The screen beneath a route with no answer
 * is replaced the moment one arrives.
 */
public class WaitingView extends RefLayout {
    private final BarView bar;
    private final Words words = new Words();

    public WaitingView(Context context, Art.Source pictures) {
        super(context);
        setBackgroundColor(Ink.BACK);
        bar = new BarView(context, pictures);
        addView(bar, RefLayout.at(0, 0, Ref.W, Ref.H));
    }

    /** The destination, lit, as the reference lights it. Nothing is lit when nothing is known. */
    public void show(Feed feed, Map<String, String> said) {
        if (words.read(said)) {
            words.into(this);
        }
        bar.setVisibility(feed.tabs.isEmpty() ? GONE : VISIBLE);
        if (!feed.tabs.isEmpty()) {
            // -1, which is `Waiting.kt:42`'s own argument: the bar shows where the page is going
            // and the remote is on nothing, because nothing on this screen answers a key yet.
            bar.show(feed, -1);
        }
    }

    public void mirror(boolean rtl) {
        setMirrored(rtl);
        bar.setMirrored(rtl);
    }
}
