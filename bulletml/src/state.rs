//! Port of C++ `BulletMLState`: the bundle the runner hands to the host via
//! [`AppRunner::create_bullet`](crate::AppRunner::create_bullet) so a child bullet
//! can resume a set of actions with the right parameters. The host passes it back
//! to [`BulletMLRunner::from_state`](crate::BulletMLRunner::from_state).

use crate::parser::BulletMLParser;
use crate::tree::NodeId;
use std::rc::Rc;

/// Opaque carrier of "where to resume and with what parameters" for a new bullet.
pub struct BulletMLState {
    // The originating parser. Sound because parsers outlive every runner/state
    // (loaded once and kept for the program's lifetime); see `BulletMLRunner`.
    pub(crate) parser: *const BulletMLParser,
    pub(crate) nodes: Vec<NodeId>,
    pub(crate) params: Option<Rc<Vec<f64>>>,
}
