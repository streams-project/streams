package streams.tv.parts;

import android.content.Context;
import android.graphics.Color;
import android.view.Gravity;
import android.view.View;
import android.widget.LinearLayout;
import android.widget.TextView;
import streams.shared.Ink;
import streams.shared.Says;
import streams.shared.Type;
import streams.tv.views.Ref;
import streams.tv.views.Round;

/**
 * The line of facts about a title, in one order on every screen: genre, year, length, age. The
 * Views twin of {@code parts/Facts.kt:52-62}.
 *
 * <h2>The order is the words', not this view's</h2>
 * {@code Facts.kt:38-50} builds the list -- first genre, year as digits in the viewer's numerals,
 * then seasons or length, then the age mark translated through {@code age.{label}} -- out of a
 * {@code contract.TitleDetail} and {@code shared/Theme.kt}'s counted forms. Every one of those is a
 * word or a contract read, so the list arrives made and this draws it. A view that re-derived the
 * order would be a second place the order lives, which is the one thing
 * {@code reference/netflix-on-the-tv.md} cannot check and a reader would notice first.
 *
 * <h2>The dot is decoration and says so</h2>
 * It carries no word, so an unmarked one is a node a reader stops on and reads out as nothing.
 */
public final class Facts extends LinearLayout {
    /** The dot between two facts. {@code Facts.kt:58}. */
    static final int DOT = 6;

    /** The room each side of it. {@code Facts.kt:58}. */
    static final int DOT_SIDE = 14;

    /** The size of a fact. {@code Facts.kt:60}. */
    static final int TYPE = 24;

    public Facts(Context context) {
        super(context);
        setOrientation(HORIZONTAL);
        setGravity(Gravity.CENTER_VERTICAL);
    }

    /** Draws these facts, in the order they are given. */
    public void show(String[] said) {
        removeAllViews();
        if (said != null) {
            for (String word : said) {
                if (word != null && !word.isEmpty()) {
                    add(word);
                }
            }
        }
        setVisibility(getChildCount() == 0 ? GONE : VISIBLE);
    }

    private void add(String word) {
        Context context = getContext();
        if (getChildCount() > 0) {
            View dot = new View(context);
            dot.setBackground(Round.round(Ink.MUTED));
            Says.onto(dot, Says.decoration());
            LayoutParams round = new LayoutParams(Ref.px(context, DOT), Ref.px(context, DOT));
            round.leftMargin = Ref.px(context, DOT_SIDE);
            round.rightMargin = Ref.px(context, DOT_SIDE);
            addView(dot, round);
        }
        TextView fact = Type.line(context, TYPE, Type.BOLD, Color.WHITE);
        fact.setText(word);
        addView(fact);
    }
}
