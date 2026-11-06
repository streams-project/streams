"""Grouping numbers. The defaults are the full catalogue's; a small set scales them down."""

from __future__ import annotations

from dataclasses import dataclass

FULL_SIZE = 20000


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
    def for_titles(cls, n: int) -> Params:
        if n >= FULL_SIZE:
            return cls()
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
