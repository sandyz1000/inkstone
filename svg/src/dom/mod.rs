use std::sync::Arc;
use std::collections::HashMap;
use roxmltree::NodeType;
pub use roxmltree::Node;

// Declare all submodules first
#[macro_use]
mod macros;
pub mod error;
pub mod util;

// These need to be after error and util since they depend on them
mod parser;
mod value;
mod attrs;
mod animate;
mod paint;
mod gradient;
mod ellipse;
mod filter;
mod g;
mod path;
mod polygon;
mod rect;
mod svg;
mod text;

// Re-export commonly used items from submodules
pub use error::Error;
pub use util::{
    Parse,
    deg2rad,
    skew_x,
    skew_y,
    transform_list,
    LengthX,
    LengthY,
    Rect as DomRect,
    Vector,
    OneOrMany,
    Iri,
    Axis,
    parse_attr,
    parse_attr_or,
    get_attr,
    style_list,
    href,
};
pub use value::{ Value, ValueVector };
pub use attrs::Attrs;
pub use animate::{ Animate, CalcMode, AnimationMode, TransformAnimate };
pub use paint::{ Fill, Stroke, Paint, Color };
pub use gradient::{ TagLinearGradient, TagRadialGradient, TagStop };
pub use ellipse::{ TagCircle, TagEllipse };
pub use filter::{ TagFilter };
pub use g::{ TagG, TagUse, TagSymbol };
pub use path::{ TagPath };
pub use polygon::{ TagPolygon, TagPolyline };
pub use rect::{ TagRect };
pub use svg::TagSvg;
pub use text::{ TagText, TagTSpan, TagTRef };

// Type alias for item collections
pub type ItemCollection = HashMap<String, Arc<Item>>;

// Define prelude after modules are declared
pub mod prelude {
    pub use pathfinder_geometry::{
        vector::{ Vector2F, vec2f },
        transform2d::Transform2F,
        rect::RectF,
    };

    pub use crate::dom::{
        Tag,
        ParseNode,
        TagDefs,
        Item,
        Error,
        Parse,
        Node,
        ItemCollection,
        Value,
        ValueVector,
        Attrs,
        Fill,
        Stroke,
        Paint,
        Animate,
        LengthX,
        LengthY,
        DomRect,
        Vector,
        Iri,
        Color,
        OneOrMany,
        Axis,
        CalcMode,
        AnimationMode,
        TransformAnimate,
        TagLinearGradient,
        TagRadialGradient,
        TagStop,
        TagCircle,
        TagEllipse,
        TagFilter,
        TagG,
        TagUse,
        TagSymbol,
        TagPath,
        TagPolygon,
        TagPolyline,
        TagRect,
        TagSvg,
        TagText,
        TagTSpan,
        TagTRef,
        deg2rad,
        skew_x,
        skew_y,
        transform_list,
    };

    pub use svgtypes::{ Length, LengthUnit };
    pub use std::str::FromStr;
}

// Re-export prelude items at module level for convenience
pub use prelude::*;
macro_rules! items {
    (
        $(#[$meta:meta])*
        pub enum $name:ident { $($($e:tt)|* => $variant:ident($data:ty),)* }
        { $($other:ident($other_data:ty),)* }
    ) => {
        $( #[$meta] )*
        pub enum $name {
            $( $variant($data), )*
            $( $other($other_data), )*
        }
        impl Tag for $name {
            fn id(&self) -> Option<&str> {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.id(), )*
                    _ => None,
                }
            }
            fn children(&self) -> &[Arc<Item>] {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.children(), )*
                    _ => &[]
                }
            }
        }
        fn parse_element(node: &Node) -> Result<Option<Item>, Error> {
            //println!("<{:?}:{} id={:?}, ...>", node.tag_name().namespace(), node.tag_name().name(), node.attribute("id"));
            let item = match node.tag_name().name() {
                $( $($e )|* => Item::$variant(<$data>::parse_node(node)?), )*
                tag => {
                    println!("unimplemented: {}", tag);
                    return Ok(None);
                }
            };
            Ok(Some(item))
        }
    };
}

items!(
    #[derive(Debug)]
    pub enum Item {
        "path" => Path(TagPath),
        "g" => G(TagG),
        "defs" => Defs(TagDefs),
        "rect" => Rect(TagRect),
        "polygon" => Polygon(TagPolygon),
        "polyline" => Polyline(TagPolyline),
        "line" => Line(TagLine),
        "circle" => Circle(TagCircle),
        "ellipse" => Ellipse(TagEllipse),
        "linearGradient" => LinearGradient(TagLinearGradient),
        "radialGradient" => RadialGradient(TagRadialGradient),
        "clipPath" => ClipPath(TagClipPath),
        "filter" => Filter(TagFilter),
        "svg" => Svg(TagSvg),
        "use" => Use(TagUse),
        "symbol" => Symbol(TagSymbol),
        "text" => Text(TagText),
        "tspan" => TSpan(TagTSpan),
        "tref" => TRef(TagTRef),
    }
    {
        String(String),
    }
);

pub trait ParseNode: Sized {
    fn parse_node(node: &Node) -> Result<Self, Error>;
}

pub trait Tag: std::fmt::Debug {
    fn id(&self) -> Option<&str> {
        None
    }
    fn children(&self) -> &[Arc<Item>] {
        &[]
    }
}

#[derive(Debug)]
pub struct TagDefs {
    items: Vec<Arc<Item>>,
}
impl Tag for TagDefs {
    fn children(&self) -> &[Arc<Item>] {
        &self.items
    }
}
impl ParseNode for TagDefs {
    fn parse_node(node: &Node) -> Result<TagDefs, Error> {
        let items = parse_node_list(node.children())?;
        Ok(TagDefs { items })
    }
}

pub fn link(ids: &mut ItemCollection, item: &Arc<Item>) {
    if let Some(id) = item.id() {
        ids.insert(id.into(), item.clone());
    }
    for child in item.children() {
        link(ids, child);
    }
}

pub fn parse_node(node: &Node, first: bool, last: bool) -> Result<Option<Item>, Error> {
    match node.node_type() {
        NodeType::Element => parse_element(node),
        NodeType::Text => parse_text(node, first, last),
        _ => Ok(None),
    }
}

fn parse_text(node: &Node, first: bool, last: bool) -> Result<Option<Item>, Error> {
    Ok(
        node.text().and_then(|s| {
            let mut last_is_space = first;
            let mut processed: String = s
                .chars()
                .filter_map(|c| {
                    if last_is_space {
                        match c {
                            '\n' | '\t' | ' ' => None,
                            _ => {
                                last_is_space = false;
                                Some(c)
                            }
                        }
                    } else {
                        match c {
                            '\n' => None,
                            '\t' | ' ' => {
                                last_is_space = true;
                                Some(' ')
                            }
                            c => Some(c),
                        }
                    }
                })
                .collect();
            if last && last_is_space && processed.len() > 0 {
                processed.pop();
            }
            if processed.len() > 0 {
                Some(Item::String(processed))
            } else {
                None
            }
        })
    )
}

pub fn parse_node_list<'a, 'i: 'a>(
    nodes: impl Iterator<Item = Node<'a, 'i>>
) -> Result<Vec<Arc<Item>>, Error> {
    let mut items = Vec::new();
    for (first, last, node) in first_or_last_node(nodes) {
        match node.node_type() {
            NodeType::Element => {
                if let Some(item) = parse_node(&node, first, last)? {
                    items.push(Arc::new(item));
                }
            }
            _ => {}
        }
    }
    Ok(items)
}

// (first, last, node)
pub fn first_or_last_node<'a, 'i: 'a>(
    nodes: impl Iterator<Item = Node<'a, 'i>>
) -> impl Iterator<Item = (bool, bool, Node<'a, 'i>)> {
    let mut nodes = nodes.enumerate().peekable();
    std::iter::from_fn(move || nodes.next().map(|(i, node)| (i == 0, nodes.peek().is_none(), node)))
}
