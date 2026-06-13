//! The BulletML node tree. Port of `bulletmltree.{h,cpp}` and the generic
//! `tree.h`, reworked as an index-based arena: nodes live in a `Vec<Node>` owned
//! by the [`BulletMLParser`](crate::BulletMLParser) and reference each other by
//! [`NodeId`], avoiding the raw parent/child pointers of the C++ version.

use crate::formula::Expr;

/// Index of a [`Node`] within the parser's arena.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Element kind — the `<tag>` name. Mirrors C++ `BulletMLNode::Name`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Name {
    Bullet,
    Action,
    Fire,
    ChangeDirection,
    ChangeSpeed,
    Accel,
    Wait,
    Repeat,
    BulletRef,
    ActionRef,
    FireRef,
    Vanish,
    Horizontal,
    Vertical,
    Term,
    Times,
    Direction,
    Speed,
    Param,
    Bulletml,
}

/// The `type=""` attribute. Mirrors C++ `BulletMLNode::Type`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Type {
    None,
    Aim,
    Absolute,
    Relative,
    Sequence,
}

/// A single element in the tree.
pub struct Node {
    pub name: Name,
    pub kind: Type,
    /// Resolved label id for `bulletRef`/`actionRef`/`fireRef`; -1 otherwise.
    pub ref_id: i32,
    /// Parsed formula from the element's text, if any.
    pub value: Option<Expr>,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
}

impl Node {
    pub fn new(name: Name) -> Self {
        Node {
            name,
            kind: Type::None,
            ref_id: -1,
            value: None,
            children: Vec::new(),
            parent: None,
        }
    }
}

/// Map an XML tag name to a [`Name`]. Returns `None` for unknown tags (the C++
/// parser asserts in that case).
pub fn name_from_str(s: &str) -> Option<Name> {
    Some(match s {
        "bulletml" => Name::Bulletml,
        "bullet" => Name::Bullet,
        "action" => Name::Action,
        "fire" => Name::Fire,
        "changeDirection" => Name::ChangeDirection,
        "changeSpeed" => Name::ChangeSpeed,
        "accel" => Name::Accel,
        "vanish" => Name::Vanish,
        "wait" => Name::Wait,
        "repeat" => Name::Repeat,
        "direction" => Name::Direction,
        "speed" => Name::Speed,
        "horizontal" => Name::Horizontal,
        "vertical" => Name::Vertical,
        "term" => Name::Term,
        "bulletRef" => Name::BulletRef,
        "actionRef" => Name::ActionRef,
        "fireRef" => Name::FireRef,
        "param" => Name::Param,
        "times" => Name::Times,
        _ => return None,
    })
}

/// Map a `type=""` attribute value to a [`Type`]. Returns `None` for unknown
/// values (the C++ parser asserts in that case).
pub fn type_from_str(s: &str) -> Option<Type> {
    Some(match s {
        "aim" => Type::Aim,
        "absolute" => Type::Absolute,
        "relative" => Type::Relative,
        "sequence" => Type::Sequence,
        _ => return None,
    })
}
