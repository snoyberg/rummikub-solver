pub mod tiles;

use self::tiles::*;
use std::fmt::Write;

struct RenderedSolution(String);

#[derive(Debug)]
struct Solution<'a> {
    rest: Option<&'a Solution<'a>>,
    current: Tiles,
}

fn solve(tiles: Tiles) -> Vec<RenderedSolution> {
    let mut res = vec![];
    solve_loop(&mut res, tiles, Tile::min_value(), None);
    res
}

fn render_solution<'a>(mut solution: Option<&'a Solution<'a>>, leftover_jokers: u8) -> RenderedSolution {
    let mut s = String::new();
    while let Some(sol) = solution {
        solution = sol.rest;
        write!(s, "({}), ", sol.current).unwrap();
    }
    write!(s, "leftover jokers: {}", leftover_jokers).unwrap();
    RenderedSolution(s)
}

fn solve_loop<'a>(results: &mut Vec<RenderedSolution>, tiles: Tiles, mut next: Tile, rest: Option<&'a Solution<'a>>) {
    loop {
        let next_count = tiles.get_count(&next);
        if next_count == 0 {
            match next.next() {
                Some(x) => {
                    next = x;
                    continue;
                }
                None => {
                    results.push(render_solution(rest, 0));
                    break;
                }
            }
        }

        let (rank, color) = match next {
            Tile::Joker => {
                // not quite a solution, but we want to know about it
                results.push(render_solution(rest, tiles.get_count(&next)));
                break;
            },
            Tile::Number(rank, color) => (rank, color),
        };

        let mut test_combo = |mut combo: Tiles| {
            let mut tiles = tiles;

            // must have at least 2 natural tiles per combo
            let mut natural = 0;

            for tile in Tile::all_no_jokers() {
                assert!(combo.get_count(&tile) <= 1);
                if combo.get_count(&tile) == 0 { continue };

                match tiles.get_count(&tile) {
                    0 => {
                        // check for a joker
                        match tiles.get_count(&Tile::Joker) {
                            0 => return false, // nothing, give up
                            count => {
                                tiles.set_count(&Tile::Joker, count - 1);
                                combo.add_tile(&Tile::Joker).unwrap();
                                combo.set_count(&tile, 0);
                            }
                        }
                    }
                    count => {
                        tiles.set_count(&tile, count - 1);
                        natural += 1;
                    }
                }
            }
            if natural < 2 { return false; }

            let solution = Solution {
                current: combo,
                rest,
            };
            solve_loop(results, tiles, next, Some(&solution));
            true
        };

        // runs
        if rank <= 11 {
            let mut combo = Tiles::new();
            combo.set_count(&next, 1);
            combo.set_count(&Tile::Number(rank + 1, color), 1);
            for rank in rank + 2 ..= 13 {
                combo.set_count(&Tile::Number(rank, color), 1);
                if !test_combo(combo) { break }
            }
        }

        // triples/quads
        let mut helper = |colors: &[Color]| {
            let mut combo = Tiles::new();
            combo.set_count(&next, 1);
            for color in colors {
                combo.set_count(&Tile::Number(rank, *color), 1);
            }
            test_combo(combo);
        };
        match color {
            Color::Black => {
                helper(&[Color::Orange, Color::Red]);
                helper(&[Color::Blue, Color::Red]);
                helper(&[Color::Blue, Color::Orange]);
                helper(&[Color::Blue, Color::Orange, Color::Red]);
            },
            Color::Blue => {
                helper(&[Color::Orange, Color::Red]);
            },
            Color::Orange => {
                // Seems like we shouldn't have to do anything here,
                // since we know we have, at most, an orange and a
                // red. However, there may still be a joker. So: test
                // out with a blue and a red, and the test_combo
                // closure above will replace the blue with a joker
                // (if available).
                //
                // We could replace blue with black, it will do the
                // same thing.
                helper(&[Color::Blue, Color::Red]);
            },
            Color::Red => ()
        }

        break;
    }
}

fn main() -> Result<(), TilesError> {
    for arg in std::env::args().skip(1) {
        let tiles = arg.parse::<Tiles>()?;
        println!("Trying to solve board: {}", tiles);
        for solution in solve(tiles) {
            println!("Solution: {}", solution.0);
        }
        println!("* * *");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn count_solutions(tiles: Tiles) -> usize {
        solve(tiles).len()
    }

    #[test]
    fn test_empty_board() {
        assert_eq!(count_solutions(Tiles::new()), 1);
    }

    #[test]
    fn test_single_tile() {
        for tile in Tile::all_no_jokers() {
            let mut tiles = Tiles::new();
            tiles.add_tile(&tile).unwrap();
            assert_eq!(count_solutions(tiles), 0);
        }
    }

    #[test]
    fn test_simple_run() {
        assert_eq!(count_solutions("1R 2R 3R".parse().unwrap()), 1);
    }

    #[test]
    fn test_run_must_be_same_color() {
        assert_eq!(count_solutions("1R 2u 3R".parse().unwrap()), 0);
    }

    #[test]
    fn test_longer_run() {
        assert_eq!(count_solutions("1R 2R 3R 4R".parse().unwrap()), 1);
    }

    #[test]
    fn test_double_run() {
        assert_eq!(count_solutions("1R 2R 3R 4R 5r 6r".parse().unwrap()), 2);
    }

    #[test]
    fn test_simple_triple() {
        assert_eq!(count_solutions("1R 1b 1u".parse().unwrap()), 1);
    }

    #[test]
    fn test_simple_triple_no_black() {
        assert_eq!(count_solutions("1R 1o 1u".parse().unwrap()), 1);
    }

    #[test]
    fn test_four() {
        assert_eq!(count_solutions("1R 1b 1u 1o".parse().unwrap()), 1);
    }

    #[test]
    fn test_must_be_different_colors() {
        assert_eq!(count_solutions("1R 1r 1u 1o".parse().unwrap()), 0);
    }

    #[test]
    fn test_can_use_jokers_triple() {
        assert_eq!(count_solutions("1R j 1o".parse().unwrap()), 1);
    }

    #[test]
    fn test_can_use_jokers_run() {
        assert_eq!(count_solutions("1R j 3r 4r".parse().unwrap()), 1);
    }

    #[test]
    fn test_need_two_natural_tiles() {
        for tile in Tile::all_no_jokers() {
            let mut tiles = "j j".parse::<Tiles>().unwrap();
            tiles.add_tile(&tile).unwrap();
            assert_eq!(count_solutions(tiles), 0);
        }
    }
}
