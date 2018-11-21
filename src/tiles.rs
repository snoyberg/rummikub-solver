use std::fmt::Display;
use std::str::FromStr;

/// The four colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Black,
    Blue,
    Orange,
    Red,
}

impl Display for Color {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                Color::Black => 'B',
                Color::Blue => 'U',
                Color::Orange => 'O',
                Color::Red => 'R',
            }
        )
    }
}

impl Color {
    fn min_value() -> Color {
        Color::Black
    }

    fn next(&self) -> Option<Color> {
        match self {
            Color::Black => Some(Color::Blue),
            Color::Blue => Some(Color::Orange),
            Color::Orange => Some(Color::Red),
            Color::Red => None,
        }
    }
}

/// Represent a single tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Joker,
    Number(u8, Color),
}

impl Tile {
    /// Get the lowest valued tile
    pub fn min_value() -> Tile {
        Tile::Number(1, Color::min_value())
    }

    /// Get the next valued tile
    pub fn next(&self) -> Option<Tile> {
        match self {
            Tile::Joker => None,
            Tile::Number(rank, color) => Some(match color.next() {
                Some(color) => Tile::Number(*rank, color),
                None => {
                    if *rank < 13 {
                        Tile::Number(rank + 1, Color::min_value())
                    } else {
                        Tile::Joker
                    }
                }
            }),
        }
    }

    /// Iterate over all the tiles
    pub fn all() -> AllTiles {
        AllTiles {
            next: Some(Tile::min_value()),
        }
    }

    /// Iterate over all the tiles except jokers
    pub fn all_no_jokers() -> AllTilesNoJokers {
        AllTilesNoJokers {
            next: Some(Tile::min_value()),
        }
    }

    /// Internal: an index in the Tiles structure for this
    /// Tile. Should be 2 greater than the previous one.
    fn index(&self) -> u8 {
        match self {
            Tile::Joker => 104,
            Tile::Number(rank, color) => {
                let color_index = match color {
                    Color::Black => 0,
                    Color::Blue => 1,
                    Color::Orange => 2,
                    Color::Red => 3,
                };
                ((rank - 1) * 4 + color_index) * 2
            }
        }
    }
}

impl FromStr for Tile {
    type Err = TilesError;

    fn from_str(s: &str) -> Result<Tile, Self::Err> {
        let get_res = || {
            if s.is_empty() {
                return None;
            }
            if s == "J" || s == "j" {
                return Some(Tile::Joker);
            }

            let bytes = s.as_bytes();
            let color = match bytes[bytes.len() - 1] {
                b'B' | b'b' => Color::Black,
                b'U' | b'u' => Color::Blue,
                b'O' | b'o' => Color::Orange,
                b'R' | b'r' => Color::Red,
                _ => return None,
            };

            let mut rank = 0;
            for b in bytes[..bytes.len() - 1].iter() {
                if *b < b'0' || *b > b'9' {
                    return None;
                }
                rank *= 10;
                rank += *b - b'0';
            }

            if rank < 1 || rank > 13 {
                None
            } else {
                Some(Tile::Number(rank, color))
            }
        };

        match get_res() {
            None => Err(TilesError::InvalidTileString(String::from(s))),
            Some(tile) => Ok(tile),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Tile::Joker => write!(fmt, "J "),
            Tile::Number(rank, color) => write!(fmt, "{}{} ", rank, color),
        }
    }
}

/// Iterator for all the tiles
pub struct AllTiles {
    next: Option<Tile>,
}

impl Iterator for AllTiles {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        let res: Tile = self.next?;
        self.next = res.next();
        Some(res)
    }
}

/// Iterator for all the tiles except jokers
pub struct AllTilesNoJokers {
    next: Option<Tile>,
}

impl Iterator for AllTilesNoJokers {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        let res: Tile = self.next?;
        self.next = match res.next() {
            None => unreachable!(),
            Some(Tile::Joker) => None,
            Some(x) => Some(x),
        };
        Some(res)
    }
}

/// Represent the tiles available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tiles {
    tiles: u128,
}

impl Tiles {
    /// An empty set of tiles
    pub fn new() -> Tiles {
        Tiles { tiles: 0 }
    }

    /// Add another tile of the given type. May fail if we already have 2.
    pub fn add_tile(&mut self, tile: &Tile) -> Result<(), TilesError> {
        let count = self.get_count(tile);
        if count >= 2 {
            Err(TilesError::AlreadyHaveTwo(*tile))
        } else {
            self.set_count(tile, count + 1);
            Ok(())
        }
    }

    /// How many of this kind of tile do we have?
    pub fn get_count(&self, tile: &Tile) -> u8 {
        let index_x = tile.index();
        let index_y = index_x + 1;

        fn is_set(x: &u128, index: u8) -> bool {
            let mask = 1 << index;
            x & mask != 0
        }

        let x = is_set(&self.tiles, index_x);
        let y = is_set(&self.tiles, index_y);

        match (y, x) {
            (false, false) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (true, true) => panic!("Not allowed: two trues"),
        }
    }

    /// Set the number of tiles of this kind. This will panic if you give it a count greater than 2.
    pub fn set_count(&mut self, tile: &Tile, count: u8) {
        let (y, x) = match count {
            0 => (false, false),
            1 => (false, true),
            2 => (true, false),
            _ => panic!("Cannot take a tile count greater than 2"),
        };

        let index_x = tile.index();
        let index_y = index_x + 1;

        fn set(x: &mut u128, index: u8, b: bool) {
            let mask = 1 << index;
            if b {
                *x |= mask;
            } else {
                *x &= !mask;
            }
        }

        set(&mut self.tiles, index_x, x);
        set(&mut self.tiles, index_y, y);
    }

    /// How many tiles total do we have?
    pub fn get_total_count(&self) -> u8 {
        Tile::all()
            .map(|tile| self.get_count(&tile))
            .fold(0, |x, y| x + y)
    }
}

impl FromStr for Tiles {
    type Err = TilesError;

    fn from_str(s: &str) -> Result<Tiles, Self::Err> {
        let mut tiles = Tiles::new();

        for s in s.split(' ') {
            let s = s.trim();
            if !s.is_empty() {
                let tile = s.parse()?;
                tiles.add_tile(&tile)?;
            }
        }

        Ok(tiles)
    }
}

impl Display for Tiles {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for tile in Tile::all() {
            for _ in 0..self.get_count(&tile) {
                tile.fmt(fmt)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TilesError {
    AlreadyHaveTwo(Tile),
    InvalidTileString(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tile_count() {
        assert_eq!(Tile::all().count(), 53)
    }

    #[test]
    fn test_has_joker() {
        assert!(Tile::all().any(|tile| tile == Tile::Joker))
    }

    #[test]
    fn test_empty_tiles() {
        assert_eq!(Tiles::new().get_total_count(), 0)
    }

    #[test]
    fn test_add_once() {
        let mut tiles = Tiles::new();
        for tile in Tile::all() {
            tiles.add_tile(&tile).unwrap();
        }
        assert_eq!(tiles.get_total_count() as usize, Tile::all().count());
    }

    #[test]
    fn test_add_twice() {
        let mut tiles = Tiles::new();
        for tile in Tile::all() {
            tiles.add_tile(&tile).unwrap();
            tiles.add_tile(&tile).unwrap();
        }
        assert_eq!(tiles.get_total_count() as usize, Tile::all().count() * 2);
    }

    #[test]
    fn test_add_thrice_fails() {
        let mut tiles = Tiles::new();
        let tile = Tile::min_value();
        assert_eq!(tiles.get_count(&tile), 0);
        assert_eq!(tiles.add_tile(&tile), Ok(()));
        assert_eq!(tiles.get_count(&tile), 1);
        assert_eq!(tiles.add_tile(&tile), Ok(()));
        assert_eq!(tiles.get_count(&tile), 2);
        assert_eq!(tiles.add_tile(&tile), Err(TilesError::AlreadyHaveTwo(tile)));
    }

    #[test]
    fn test_tile_index() {
        let mut expected = 0;
        for tile in Tile::all() {
            assert_eq!(tile.index(), expected);
            expected += 2;
        }
    }

    #[test]
    fn test_parse_empty() {
        assert_eq!("".parse::<Tiles>().unwrap(), Tiles::new());
    }

    #[test]
    fn test_parse_one_joker() {
        let mut tiles = Tiles::new();
        tiles.add_tile(&Tile::Joker).unwrap();
        assert_eq!("J".parse::<Tiles>().unwrap(), tiles);
    }

    #[test]
    fn test_parse_numbers() {
        let mut tiles = Tiles::new();
        tiles.add_tile(&Tile::Number(5, Color::Blue)).unwrap();
        tiles.add_tile(&Tile::Number(6, Color::Black)).unwrap();
        tiles.add_tile(&Tile::Number(6, Color::Black)).unwrap();
        assert_eq!("6B 5U 6B".parse::<Tiles>().unwrap(), tiles);
    }

    #[test]
    fn test_parse_too_many_jokers() {
        assert_eq!(
            "J j J".parse::<Tiles>(),
            Err(TilesError::AlreadyHaveTwo(Tile::Joker))
        );
    }

    #[test]
    fn test_parse_case() {
        let mut tiles = Tiles::new();
        tiles.add_tile(&Tile::Number(5, Color::Blue)).unwrap();
        tiles.add_tile(&Tile::Number(6, Color::Black)).unwrap();
        tiles.add_tile(&Tile::Number(6, Color::Red)).unwrap();
        tiles.add_tile(&Tile::Number(7, Color::Orange)).unwrap();
        tiles.add_tile(&Tile::Number(8, Color::Orange)).unwrap();

        assert_eq!("6b 5U 6r 7o 8O".parse::<Tiles>().unwrap(), tiles);
    }

    #[test]
    fn test_parse_display() {
        fn helper(tiles: &Tiles) {
            let s = tiles.to_string();
            assert_eq!(&s.parse::<Tiles>().unwrap(), tiles);
        }

        let mut tiles = Tiles::new();
        helper(&tiles);
        for tile in Tile::all() {
            tiles.add_tile(&tile).unwrap();
            helper(&tiles);
            tiles.add_tile(&tile).unwrap();
            helper(&tiles);
        }
    }
}
