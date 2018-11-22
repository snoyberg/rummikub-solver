#![feature(proc_macro_hygiene)]
#![feature(impl_trait_in_bindings)]
pub mod tiles;
pub mod solve;

use stdweb::web::{document, INode, IElement, IEventTarget, Element, Document, window};
use stdweb::web::event::ClickEvent;
use stdweb::web::error::InvalidCharacterError;

use std::rc::Rc;
use std::cell::RefCell;

use self::tiles::{Tiles, Color, Tile};
use self::solve::{solve};

fn main() {
    build().unwrap();
}

fn build() -> Result<(), Box<std::error::Error>> {
    let doc = document();

    let head = match doc.head() {
        None => return Err(From::from("Could not find head element")),
        Some(x) => x,
    };
    let link = doc.create_element("link")?;
    link.set_attribute("rel", "stylesheet")?;
    link.set_attribute("href", "https://stackpath.bootstrapcdn.com/bootstrap/4.1.3/css/bootstrap.min.css")?;
    link.set_attribute("integrity", "sha384-MCw98/SFnGE8fJT3GXwEOngsV7Zt27NXFoaoApmYm81iuXoPkFOJwJ8ERdknLPMO")?;
    link.set_attribute("crossorigin", "anonymous")?;
    head.append_child(&link);

    let style = doc.create_element("style")?;
    let css = r#"
td.rank {
    text-align: right;
    padding-left: 1.5em;
}
.tile {
    display: inline-block;
    border: 1px solid #999;
    background-color: #eee;
    padding: 0 3px;
    width: 1.5em;
    text-align: center;
}
.tile.red {
    color: red;
}
.tile.orange {
    color: orange;
}
.tile.blue {
    color: blue;
}
.combo {
    margin-right: 1em;
}
ul.solutions {
    list-style: none;
    padding: 0;
}
ul.solutions > li:first-child {
    border-top: 0;
    margin-top: 0;
    padding-top: 0;
}
ul.solutions > li {
    margin-top: 0.5em;
    padding-top: 0.5em;
    border-top: 1px solid black;
}
"#;
    style.append_child(&doc.create_text_node(css));
    head.append_child(&style);

    let body = match doc.body() {
        None => return Err(From::from("Could not find body element")),
        Some(x) => x,
    };

    let container = doc.create_element("div")?;
    container.set_attribute("class", "container")?;
    body.append_child(&container);

    let row = doc.create_element("div")?;
    row.set_attribute("class", "row")?;
    container.append_child(&row);

    let board_column = doc.create_element("div")?;
    board_column.set_attribute("class", "col-lg-9")?;
    row.append_child(&board_column);

    let sol_column = doc.create_element("div")?;
    sol_column.set_attribute("class", "col-lg-3")?;
    row.append_child(&sol_column);

    let solutions_h1 = doc.create_element("h1")?;
    solutions_h1.set_attribute("class", "text-center")?;
    solutions_h1.append_child(&doc.create_text_node("Solutions"));
    sol_column.append_child(&solutions_h1);

    let solution_ul = doc.create_element("ul")?;
    solution_ul.set_attribute("class", "solutions")?;
    sol_column.append_child(&solution_ul);
    let solution_ul = Rc::new(solution_ul);

    let board_h1 = doc.create_element("h1")?;
    board_h1.set_attribute("class", "text-center")?;
    board_column.append_child(&board_h1);
    board_h1.append_child(&doc.create_text_node("Board"));

    let table = doc.create_element("table")?;
    board_column.append_child(&table);

    let tbody = doc.create_element("tbody")?;
    table.append_child(&tbody);

    let tiles = Rc::new(RefCell::new(Tiles::new()));

    let mut make_buttons: impl FnMut(&Document, &Element, Tile) -> Result<(), InvalidCharacterError> = move |doc: &Document, td: &Element, tile: Tile| {
        let button_group = doc.create_element("div")?;
        button_group.set_attribute("class", "btn-group")?;
        td.append_child(&button_group);

        const ACTIVE: &str = "btn btn-outline-primary btn-sm active";
        const INACTIVE: &str = "btn btn-outline-primary btn-sm";

        let buttons = Rc::new(
            [ doc.create_element("button")?
            , doc.create_element("button")?
            , doc.create_element("button")?
            ]);

        for (button, count) in buttons.iter().zip(0..) {
            button.set_attribute("class", if count == 0 {ACTIVE} else {INACTIVE})?;
            button.append_child(&doc.create_text_node(&count.to_string()));
            button_group.append_child(button);

            let buttons = buttons.clone();
            let tiles = tiles.clone();
            let solution_ul = solution_ul.clone();
            let callback = move || {
                let doc = document();
                for (button, innercount) in buttons.iter().zip(0..) {
                    let class = if count == innercount {ACTIVE} else {INACTIVE};
                    button.set_attribute("class", class)?;
                }

                for node in solution_ul.child_nodes() {
                    solution_ul.remove_child(&node)?;
                }

                let mut tiles = tiles.borrow_mut();
                tiles.set_count(&tile, count);
                let solutions = solve(*tiles);

                for sol in solutions.into_iter().rev().filter(|sol| sol.leftover_jokers == 0) {
                    let li = doc.create_element("li")?;
                    solution_ul.append_child(&li);

                    for combo in sol.combos {
                        let span = doc.create_element("span")?;
                        span.set_attribute("class", "combo")?;
                        li.append_child(&span);

                        for tile in Tile::all() {
                            let count = combo.get_count(&tile);
                            assert!(count < 2);
                            if count == 1 {
                                span.append_child(&tile_span(&doc, &tile)?);
                            }
                        }
                    }
                }

                let res: Result<(), Box<std::error::Error>> = Ok(());
                res
            };
            button.add_event_listener(move |_: ClickEvent| {
                match callback() {
                    Ok(()) => (),
                    Err(e) => window().alert(&format!("That was weird: {}", e)),
                }
            });
        }

        Ok(())
    };

    for rank in 1..=13 {
        let row = doc.create_element("tr")?;
        tbody.append_child(&row);

        for color in Color::all() {
            let td = doc.create_element("td")?;
            let tile = Tile::Number(rank, color);
            td.append_child(&tile_span(&doc, &tile)?);
            td.set_attribute("class", "rank")?;
            row.append_child(&td);

            let td = doc.create_element("td")?;
            make_buttons(&doc, &td, tile)?;
            row.append_child(&td);

        }
    }

    let row = doc.create_element("tr")?;
    tbody.append_child(&row);

    let td = doc.create_element("td")?;
    td.set_attribute("class", "rank")?;
    td.append_child(&tile_span(&doc, &Tile::Joker)?);
    row.append_child(&td);

    let td = doc.create_element("td")?;
    td.set_attribute("colspan", "7")?;
    row.append_child(&td);
    make_buttons(&doc, &td, Tile::Joker)?;

    Ok(())
}

fn tile_span(doc: &Document, tile: &Tile) -> Result<Element, InvalidCharacterError> {
    let span = doc.create_element("span")?;
    match tile {
        Tile::Joker => {
            span.set_attribute("class", "tile")?;
            span.append_child(&doc.create_text_node("J"));
        }
        Tile::Number(rank, color) => {
            let class = match color {
                Color::Black => "tile black",
                Color::Blue => "tile blue",
                Color::Orange => "tile orange",
                Color::Red => "tile red",
            };
            span.set_attribute("class", class)?;
            span.append_child(&doc.create_text_node(&rank.to_string()));
        }
    }
    Ok(span)
}
