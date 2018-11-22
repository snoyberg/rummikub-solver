#![feature(proc_macro_hygiene)]
#![feature(impl_trait_in_bindings)]
pub mod tiles;
pub mod solve;

use stdweb::web::{document, INode, IElement, IEventTarget, Element, Document};
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
    let style = doc.create_element("style")?;
    let css = r#"
        table {
            border-collapse: collapse;
        }
        td {
            border: 1px solid black;
            padding: 5px;
            font-size: 0.4em;
        }
        div.rank {
            font-size: 2em;
            font-weight: bold;
        }
"#;
    style.append_child(&doc.create_text_node(css));
    head.append_child(&style);

    let body = match doc.body() {
        None => return Err(From::from("Could not find body element")),
        Some(x) => x,
    };
    let table = doc.create_element("table")?;

    let tbody = doc.create_element("tbody")?;
    table.append_child(&tbody);

    let tiles = Rc::new(RefCell::new(Tiles::new()));

    let mut radio_name = 0;

    let solution = Rc::new(doc.create_element("pre")?);
    let solution_clone = solution.clone();

    let mut make_tile: impl FnMut(&Document, &Element, Tile) -> Result<(), InvalidCharacterError> = move |doc: &Document, td: &Element, tile: Tile| {
        let div = doc.create_element("div")?;
        div.set_attribute("class", "rank")?;
        div.append_child(&doc.create_text_node(&match tile {
            Tile::Joker => String::from("Joker"),
            Tile::Number(rank, _) => rank.to_string(),
        }));
        td.append_child(&div);

        // FIXME use push buttons instead
        let name = format!("radio_{}", radio_name);
        radio_name += 1;
        for count in 0..=2 {
            let input = doc.create_element("input")?;
            input.set_attribute("name", &name)?;
            input.set_attribute("type", "radio")?;
            if count == 0 { input.set_attribute("checked", "checked")?; }

            td.append_child(&input);
            td.append_child(&doc.create_text_node(&count.to_string()));

            let tiles = tiles.clone();
            let solution = solution.clone();
            input.add_event_listener(move |_: ClickEvent| {
                let mut tiles = tiles.borrow_mut();
                tiles.set_count(&tile, count);
                let solutions = solve(*tiles);
                let mut solutions_text = String::from("Valid solutions:\n");
                for sol in solutions {
                    solutions_text += "Solution: ";
                    solutions_text += &sol.to_string();
                    solutions_text += "\n";
                }
                solution.set_text_content(&solutions_text);
            });
        }

        Ok(())
    };

    for rank in 1..=13 {
        let row = doc.create_element("tr")?;
        tbody.append_child(&row);

        for color in Color::all() {
            let td = doc.create_element("td")?;
            td.set_attribute("style", &format!("color: {}", match color {
                Color::Black => "black",
                Color::Blue => "blue",
                Color::Orange => "orange",
                Color::Red => "red",
            }))?;
            make_tile(&doc, &td, Tile::Number(rank, color))?;
            row.append_child(&td);

        }
    }

    let row = doc.create_element("tr")?;
    tbody.append_child(&row);
    let td = doc.create_element("td")?;
    row.append_child(&td);
    make_tile(&doc, &td, Tile::Joker)?;

    body.append_child(&table);
    body.append_child(&*solution_clone);

    Ok(())
}
