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
    let link = doc.create_element("link")?;
    link.set_attribute("rel", "stylesheet")?;
    link.set_attribute("href", "https://stackpath.bootstrapcdn.com/bootstrap/4.1.3/css/bootstrap.min.css")?;
    link.set_attribute("integrity", "sha384-MCw98/SFnGE8fJT3GXwEOngsV7Zt27NXFoaoApmYm81iuXoPkFOJwJ8ERdknLPMO")?;
    link.set_attribute("crossorigin", "anonymous")?;
    head.append_child(&link);

    let script = doc.create_element("script")?;
    script.set_attribute("src", "https://code.jquery.com/jquery-3.3.1.slim.min.js")?;
    script.set_attribute("integrity", "sha384-q8i/X+965DzO0rT7abK41JStQIAqVgRVzpbzo5smXKp4YfRvH+8abtTE1Pi6jizo")?;
    script.set_attribute("crossorigin", "anonymous")?;
    head.append_child(&script);

    let script = doc.create_element("script")?;
    script.set_attribute("src", "https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.14.3/umd/popper.min.js")?;
    script.set_attribute("integrity", "sha384-ZMP7rVo3mIykV+2+9J3UJ46jBk0WLaUAdn689aCwoqbBJiSnjAK/l8WvCWPIPm49")?;
    script.set_attribute("crossorigin", "anonymous")?;
    head.append_child(&script);

    let script = doc.create_element("script")?;
    script.set_attribute("src", "https://stackpath.bootstrapcdn.com/bootstrap/4.1.3/js/bootstrap.min.js")?;
    script.set_attribute("integrity", "sha384-ChfqqxuZUCnJSK3+MXmPNIyE6ZbWh2IMqE241rYiqJxyMiZ6OW/JmZQ5stwEULTy")?;
    script.set_attribute("crossorigin", "anonymous")?;
    head.append_child(&script);

    let style = doc.create_element("style")?;
    let css = r#"
        td.rank {
            text-align: right;
            margin-left: 3px;
        }
/*
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
*/
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
    board_column.set_attribute("class", "col-lg-6")?;
    row.append_child(&board_column);

    let sol_column = doc.create_element("div")?;
    sol_column.set_attribute("class", "col-lg-6")?;
    row.append_child(&sol_column);

    let solution = doc.create_element("pre")?;
    sol_column.append_child(&solution);
    let solution = Rc::new(solution);

    let table = doc.create_element("table")?;

    let tbody = doc.create_element("tbody")?;
    table.append_child(&tbody);

    let tiles = Rc::new(RefCell::new(Tiles::new()));

    let mut radio_name = 0;

    let mut make_tile: impl FnMut(&Document, &Element, Tile) -> Result<(), InvalidCharacterError> = move |doc: &Document, td: &Element, tile: Tile| {
        let button_group = doc.create_element("div")?;
        button_group.set_attribute("class", "btn-group btn-group-toggle")?;
        button_group.set_attribute("data-toggle", "buttons")?;
        td.append_child(&button_group);
        let name = format!("radio_{}", radio_name);
        radio_name += 1;
        for count in 0..=2 {
            let label = doc.create_element("label")?;
            label.set_attribute("class",
                                if count == 0 {
                                    "btn btn-outline-primary btn-sm active"
                                } else {
                                    "btn btn-outline-primary btn-sm"
                                })?;
            button_group.append_child(&label);

            let input = doc.create_element("input")?;
            input.set_attribute("name", &name)?;
            input.set_attribute("type", "radio")?;
            if count == 0 { input.set_attribute("checked", "checked")?; }

            label.append_child(&input);
            label.append_child(&doc.create_text_node(&count.to_string()));

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
            td.set_attribute("class", "rank")?;
            td.set_attribute("style", &format!("color: {}", match color {
                Color::Black => "black",
                Color::Blue => "blue",
                Color::Orange => "orange",
                Color::Red => "red",
            }))?;
            td.append_child(&doc.create_text_node(&rank.to_string()));
            row.append_child(&td);

            let td = doc.create_element("td")?;
            make_tile(&doc, &td, Tile::Number(rank, color))?;
            row.append_child(&td);

        }
    }

    let row = doc.create_element("tr")?;
    tbody.append_child(&row);

    let td = doc.create_element("td")?;
    td.append_child(&doc.create_text_node("Joker"));
    row.append_child(&td);

    let td = doc.create_element("td")?;
    td.set_attribute("colspan", "7")?;
    row.append_child(&td);
    make_tile(&doc, &td, Tile::Joker)?;

    board_column.append_child(&table);

    Ok(())
}
