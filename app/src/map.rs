use bevy::{math::{IRect, IVec2}};
use log::warn;
use roxmltree::Node;

use crate::game::SCREEN_HEIGHT;

pub struct Map {
    pub solids: Vec<IRect>,
    pub spawn_a: IVec2,
    pub spawn_b: IVec2,
}

pub fn load_map() -> Map {
    let file = include_str!("../assets/map.svg");

    let doc = roxmltree::Document::parse(file).unwrap();
    let mut map = Map { solids: Vec::new(), spawn_a: IVec2::ZERO, spawn_b: IVec2::ZERO };

    for node in doc.root().descendants() {
        match shape(node) {
            Ok(rect) => match node.attribute("fill") {
                Some("#FF0000") => map.solids.push(rect),
                Some("#0000FA") => map.spawn_a = rect.min,
                Some("#0000FB") => map.spawn_b = rect.min,
                _ => warn!("no valid fill attr"),
            }
            Err(error) => {
                warn!("{}", error);
            }
        }
    }

    map
}

pub fn shape(node: Node) -> Result<IRect, &'static str> {
    match node.tag_name().name() {
        "rect" => {
            let x: i32 = node
                .attribute("x").unwrap_or("0")
                .parse()
                .or(Err("invalid attribute x"))?;
            let y: i32 = node
                .attribute("y").unwrap_or("0")
                .parse()
                .or(Err("invalid attribute y"))?;
            let width: i32 = node
                .attribute("width").unwrap_or("0")
                .parse()
                .or(Err("invalid attribute width"))?;
            let height: i32 = node
                .attribute("height").unwrap_or("0")
                .parse()
                .or(Err("invalid attribute height"))?;

            // convert svg y-down to y-up
            Ok(IRect {
                min: IVec2 { x, y: SCREEN_HEIGHT as i32 - (y + height) },
                max: IVec2 { x: x + width, y: SCREEN_HEIGHT as i32 - y },
            })
        }
        _ => Err("invalid tag name"),
    }
}
