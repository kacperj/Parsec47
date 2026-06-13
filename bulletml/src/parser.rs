//! BulletML parser. Port of `bulletmlparser.{h,cpp}` and
//! `bulletmlparser-tinyxml.cpp`, using `roxmltree` in place of bundled TinyXML.
//!
//! Builds the immutable node arena, records the `horizontal` orientation, and
//! resolves `label`/`*Ref` attributes into per-domain integer ids (the C++
//! `IDPool`) so the runner can look a reference's target node up by id.

use crate::formula;
use crate::tree::{name_from_str, type_from_str, Name, Node, NodeId};
use std::collections::HashMap;
use std::fmt;

/// Error returned by [`BulletMLParser::parse_file`] / [`parse_str`](BulletMLParser::parse_str).
#[derive(Debug)]
pub enum ParseError {
    /// Failed to read the file.
    Io(std::io::Error),
    /// The XML was malformed.
    Xml(roxmltree::Error),
    /// The XML was well-formed but not valid BulletML (bad tag, type, or shape).
    Format(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "I/O error: {e}"),
            ParseError::Xml(e) => write!(f, "XML error: {e}"),
            ParseError::Format(s) => write!(f, "BulletML error: {s}"),
        }
    }
}

impl std::error::Error for ParseError {}

// Per-domain label -> id pool (C++ `IDPool`). Domains are bullet/action/fire.
#[derive(Default)]
struct IdPool {
    maps: HashMap<Name, HashMap<String, i32>>,
    next: HashMap<Name, i32>,
}

impl IdPool {
    fn get_id(&mut self, domain: Name, key: &str) -> i32 {
        let map = self.maps.entry(domain).or_default();
        if let Some(&id) = map.get(key) {
            id
        } else {
            let n = self.next.entry(domain).or_insert(0);
            let id = *n;
            *n += 1;
            map.insert(key.to_string(), id);
            id
        }
    }
}

/// A parsed BulletML document: the node arena plus reference maps. Long-lived and
/// immutable once built; [`BulletMLRunner`](crate::BulletMLRunner)s borrow it.
pub struct BulletMLParser {
    arena: Vec<Node>,
    horizontal: bool,
    top_actions: Vec<NodeId>,
    // label-id -> node, per domain.
    bullet_map: Vec<Option<NodeId>>,
    action_map: Vec<Option<NodeId>>,
    fire_map: Vec<Option<NodeId>>,
}

impl BulletMLParser {
    /// Parse a BulletML document from a file path.
    pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ParseError> {
        let text = std::fs::read_to_string(path).map_err(ParseError::Io)?;
        Self::parse_str(&text)
    }

    /// Parse a BulletML document from an in-memory string.
    pub fn parse_str(xml: &str) -> Result<Self, ParseError> {
        // BulletML files carry a `<!DOCTYPE ... SYSTEM "...bulletml.dtd">`; allow it
        // (the DTD is not fetched or validated, matching TinyXML's behaviour).
        let opts = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc = roxmltree::Document::parse_with_options(xml, opts).map_err(ParseError::Xml)?;
        let mut parser = BulletMLParser {
            arena: Vec::new(),
            horizontal: false,
            top_actions: Vec::new(),
            bullet_map: Vec::new(),
            action_map: Vec::new(),
            fire_map: Vec::new(),
        };
        let mut pool = IdPool::default();
        let root = doc.root_element();
        if name_from_str(root.tag_name().name()) != Some(Name::Bulletml) {
            return Err(ParseError::Format("root element is not <bulletml>".into()));
        }
        parser.walk(root, None, &mut pool)?;
        Ok(parser)
    }

    // Port of getTree/translateNode + addContent/addAttribute, fused into one
    // recursive walk over the roxmltree.
    fn walk(
        &mut self,
        elem: roxmltree::Node,
        parent: Option<NodeId>,
        pool: &mut IdPool,
    ) -> Result<(), ParseError> {
        let tag = elem.tag_name().name();
        let name = name_from_str(tag)
            .ok_or_else(|| ParseError::Format(format!("unknown tag <{tag}>")))?;

        let id = NodeId(self.arena.len());
        self.arena.push(Node::new(name));
        self.arena[id.0].parent = parent;

        if name == Name::Bulletml {
            // The root is not added as anyone's child and takes no labels; the only
            // attribute that matters is a value of "horizontal" (e.g. type="horizontal").
            for attr in elem.attributes() {
                if attr.value() == "horizontal" {
                    self.horizontal = true;
                }
            }
        } else {
            self.add_attributes(elem, id, name, pool)?;
            if let Some(p) = parent {
                self.arena[p.0].children.push(id);
            }
        }

        // Element text (the formula). Value nodes hold their formula as text; the
        // whitespace between child elements of container nodes is ignored.
        let text: String = elem
            .children()
            .filter(|c| c.is_text())
            .filter_map(|c| c.text())
            .collect();
        if !text.trim().is_empty() {
            self.arena[id.0].value = Some(formula::parse(&text));
        }

        for child in elem.children().filter(|c| c.is_element()) {
            self.walk(child, Some(id), pool)?;
        }
        Ok(())
    }

    // Port of addAttribute: handle `type` and `label` (id assignment + ref maps).
    fn add_attributes(
        &mut self,
        elem: roxmltree::Node,
        id: NodeId,
        name: Name,
        pool: &mut IdPool,
    ) -> Result<(), ParseError> {
        for attr in elem.attributes() {
            match attr.name() {
                "type" => {
                    let ty = type_from_str(attr.value()).ok_or_else(|| {
                        ParseError::Format(format!("unknown type \"{}\"", attr.value()))
                    })?;
                    self.arena[id.0].kind = ty;
                }
                "label" => {
                    let val = attr.value();
                    let domain = match name {
                        Name::BulletRef => Name::Bullet,
                        Name::ActionRef => Name::Action,
                        Name::FireRef => Name::Fire,
                        other => other,
                    };
                    let label_id = pool.get_id(domain, val);
                    match name {
                        Name::Bullet => set_map(&mut self.bullet_map, label_id, id),
                        Name::Action => set_map(&mut self.action_map, label_id, id),
                        Name::Fire => set_map(&mut self.fire_map, label_id, id),
                        Name::BulletRef | Name::ActionRef | Name::FireRef => {
                            self.arena[id.0].ref_id = label_id;
                        }
                        _ => {
                            return Err(ParseError::Format(
                                "element cannot have a \"label\" attribute".into(),
                            ))
                        }
                    }
                    // Actions whose label starts with "top" are entry points.
                    if name == Name::Action && val.starts_with("top") {
                        self.top_actions.push(id);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    // ---- accessors used by the runner ----

    pub(crate) fn node(&self, id: NodeId) -> &Node {
        &self.arena[id.0]
    }

    pub fn is_horizontal(&self) -> bool {
        self.horizontal
    }

    pub(crate) fn top_actions(&self) -> &[NodeId] {
        &self.top_actions
    }

    pub(crate) fn get_bullet_ref(&self, id: i32) -> NodeId {
        self.bullet_map[id as usize].expect("bulletRef target missing")
    }

    pub(crate) fn get_action_ref(&self, id: i32) -> NodeId {
        self.action_map[id as usize].expect("actionRef target missing")
    }

    pub(crate) fn get_fire_ref(&self, id: i32) -> NodeId {
        self.fire_map[id as usize].expect("fireRef target missing")
    }

    /// First child of `id` whose tag is `name`, if any.
    pub(crate) fn get_child(&self, id: NodeId, name: Name) -> Option<NodeId> {
        self.arena[id.0]
            .children
            .iter()
            .copied()
            .find(|&c| self.arena[c.0].name == name)
    }

    /// All children of `id` whose tag is `name`, appended to `out`.
    pub(crate) fn get_all_children(&self, id: NodeId, name: Name, out: &mut Vec<NodeId>) {
        for &c in &self.arena[id.0].children {
            if self.arena[c.0].name == name {
                out.push(c);
            }
        }
    }

    pub(crate) fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.arena[id.0].children.first().copied()
    }

    pub(crate) fn child_count(&self, id: NodeId) -> usize {
        self.arena[id.0].children.len()
    }

    /// The next sibling of `id` using its real parent (C++ `BulletMLNode::next`).
    /// The runner overrides this for its root nodes (whose parent it treats as none).
    pub(crate) fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        let parent = self.arena[id.0].parent?;
        let siblings = &self.arena[parent.0].children;
        let pos = siblings.iter().position(|&c| c == id)?;
        siblings.get(pos + 1).copied()
    }
}

fn set_map(map: &mut Vec<Option<NodeId>>, id: i32, node: NodeId) {
    let idx = id as usize;
    if map.len() <= idx {
        map.resize(idx + 1, None);
    }
    map[idx] = Some(node);
}
