"""Grouping numbers. The defaults are the full catalogue's; a small set scales them down."""

from __future__ import annotations

from dataclasses import dataclass

RAMP_START = 15000
RAMP_END = 25000
FULL_SIZE = RAMP_END


@dataclass(frozen=True)
class Params:
    min_text: int = 100
    k_graph: int = 15
    k_vote: int = 25
    tau: float = 0.05
    target_l1: int = 60
    min_l1: int = 200
    sub_size: int = 150
    min_split: int = 300
    min_sub: int = 30
    atom_size: int = 48
    min_atom: int = 12
    theme_targets: tuple[int, int, int] = (150, 400, 1000)
    vibe_fine_size: int = 80
    vibe_mid_size: int = 400
    vibe_mid_floor: int = 20
    min_vibe_fine: int = 15
    min_vibe_mid: int = 60
    season_size: int = 30
    min_season: int = 10
    season_whole_below: int = 48
    season_pool_floor: int = 400
    season_pool_ceiling: int = 1500

    @classmethod
    def _scaled_params(cls, n: int) -> Params:
        sub = max(8, n // 60)
        return cls(
            min_text=20,
            k_graph=min(15, max(3, n // 10)),
            k_vote=min(25, max(3, n // 10)),
            target_l1=max(3, round(n / 150)),
            min_l1=max(4, n // 100),
            sub_size=sub,
            min_split=2 * sub,
            min_sub=max(3, sub // 5),
            atom_size=max(4, n // 80),
            min_atom=2,
            theme_targets=(max(2, n // 120), max(3, n // 60), max(4, n // 30)),
            vibe_fine_size=max(5, n // 40),
            vibe_mid_size=max(10, n // 10),
            vibe_mid_floor=2,
            min_vibe_fine=2,
            min_vibe_mid=3,
            season_size=max(6, n // 60),
            min_season=2,
            season_whole_below=12,
            season_pool_floor=max(12, n // 10),
            season_pool_ceiling=max(24, n // 4),
        )

    @classmethod
    def for_titles(cls, n: int) -> Params:
        if n >= RAMP_END:
            return cls()
        scaled = cls._scaled_params(n)
        if n <= RAMP_START:
            return scaled
        t = (n - RAMP_START) / (RAMP_END - RAMP_START)
        full = cls()

        def lerp(a: int, b: int) -> int:
            return round((1.0 - t) * a + t * b)

        return cls(
            min_text=lerp(scaled.min_text, full.min_text),
            k_graph=lerp(scaled.k_graph, full.k_graph),
            k_vote=lerp(scaled.k_vote, full.k_vote),
            tau=full.tau,
            target_l1=lerp(scaled.target_l1, full.target_l1),
            min_l1=lerp(scaled.min_l1, full.min_l1),
            sub_size=lerp(scaled.sub_size, full.sub_size),
            min_split=lerp(scaled.min_split, full.min_split),
            min_sub=lerp(scaled.min_sub, full.min_sub),
            atom_size=lerp(scaled.atom_size, full.atom_size),
            min_atom=lerp(scaled.min_atom, full.min_atom),
            theme_targets=(
                lerp(scaled.theme_targets[0], full.theme_targets[0]),
                lerp(scaled.theme_targets[1], full.theme_targets[1]),
                lerp(scaled.theme_targets[2], full.theme_targets[2]),
            ),
            vibe_fine_size=lerp(scaled.vibe_fine_size, full.vibe_fine_size),
            vibe_mid_size=lerp(scaled.vibe_mid_size, full.vibe_mid_size),
            vibe_mid_floor=lerp(scaled.vibe_mid_floor, full.vibe_mid_floor),
            min_vibe_fine=lerp(scaled.min_vibe_fine, full.min_vibe_fine),
            min_vibe_mid=lerp(scaled.min_vibe_mid, full.min_vibe_mid),
            season_size=lerp(scaled.season_size, full.season_size),
            min_season=lerp(scaled.min_season, full.min_season),
            season_whole_below=lerp(scaled.season_whole_below, full.season_whole_below),
            season_pool_floor=lerp(scaled.season_pool_floor, full.season_pool_floor),
            season_pool_ceiling=lerp(scaled.season_pool_ceiling, full.season_pool_ceiling),
        )
