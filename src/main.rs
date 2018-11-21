pub mod tiles;

use self::tiles::*;

fn main() -> Result<(), TilesError> {
    for arg in std::env::args().skip(1) {
        let tiles = arg.parse::<Tiles>()?;
        println!("Trying to solve board: {}", tiles);
    }
    Ok(())
}
