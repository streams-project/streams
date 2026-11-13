package streams.ports;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import contract.Contract;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;

/**
 * Java source calling the generated contract.
 *
 * <p>This file is the proof for the gate note 210 named. The Kotlin arm of the generator writes
 * {@code TitleKey} as a {@code @JvmInline value class}, so the Kotlin {@code contract.Tile} asks
 * Java to call {@code getTitle-syqQ2Kk()} - a hyphen is no Java identifier, and the primary
 * constructor is private besides. Measured on this machine 2026-11-13 with javap over
 * build/classes/kotlin/main/contract/Tile.class.
 *
 * <p>Every type named below is one of those value class backed shapes, reached through the Java
 * arm instead: {@code contract.Contract.Tile}, {@code contract.Contract.TitleKey} and
 * {@code contract.Contract.AgeTone}. If the generated folder stops being compiled by javac, this
 * file does not compile, and the failure says so by name rather than by a missing test.
 */
class ContractFromJavaTest {

    private static final String KEY = "tv:81040344";

    @Test
    void constructsATileThroughTheValueClassBackedKey() {
        Contract.TitleKey key = new Contract.TitleKey(KEY);
        List<String> tags = Arrays.asList("resume", "top10");
        Contract.Tile tile =
                new Contract.Tile(null, null, null, "A Name", Long.valueOf(42L), tags, key);

        assertEquals("A Name", tile.getName());
        assertEquals(KEY, tile.getTitle().value());
        assertTrue(tile.hasPercent());
        assertEquals(42L, tile.getPercent());
        assertEquals(2, tile.getTagsCount());
        assertEquals("resume", tile.getTags(0));
        assertFalse(tile.hasEpisode());
        assertEquals(0, tile.getBadgesCount());
    }

    @Test
    void aTileRoundTripsThroughTheWire() {
        Contract.Tile tile =
                new Contract.Tile(
                        null,
                        null,
                        null,
                        "A Name",
                        Long.valueOf(42L),
                        Arrays.asList("resume"),
                        new Contract.TitleKey(KEY));
        String json = Contract.Json.render(tile);
        Contract.Tile back = Contract.Tile.read(Contract.Json.parse(json));

        assertEquals(tile.getName(), back.getName());
        assertEquals(tile.getTitle().value(), back.getTitle().value());
        assertEquals(tile.getPercent(), back.getPercent());
        assertEquals(tile.getTagsList(), back.getTagsList());
    }

    @Test
    void theKeyKeepsItsShapeRule() {
        assertThrows(IllegalArgumentException.class, () -> new Contract.TitleKey("tv:0"));
        assertThrows(IllegalArgumentException.class, () -> new Contract.TitleKey("book:12"));
        assertEquals("movie:1", new Contract.TitleKey("movie:1").value());
    }

    @Test
    void anAgeToneIsAValueAndCompares() {
        assertEquals("dark_red", Contract.AgeTone.DARK_RED.value());
        assertEquals(Contract.AgeTone.GREEN, new Contract.AgeTone("green"));
        assertFalse(Contract.AgeTone.GREEN.equals(Contract.AgeTone.RED));
    }
}
