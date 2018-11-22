use super::tiles::{Tiles, TilesError};
use super::solve::solve;

pub fn main() -> Result<(), TilesError> {
    for arg in std::env::args().skip(1) {
        let tiles = arg.parse::<Tiles>()?;
        println!("Trying to solve board: {}", tiles);
        for solution in solve(tiles) {
            println!("Solution: {}", solution);
        }
        println!("* * *");
    }

    Ok(())
}
