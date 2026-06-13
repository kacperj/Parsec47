//! The BulletML interpreter. Port of `bulletmlrunner.cpp` (the outer
//! [`BulletMLRunner`], one sub-runner per top action) and `bulletmlrunnerimpl.cpp`
//! (the per-action frame-stepped state machine, [`Impl`]).
//!
//! Each [`Impl`] holds a raw `*const BulletMLParser`. This is sound because a
//! parser is loaded once and outlives every runner created from it (the host
//! keeps parsers for the whole program); the runner only ever reads the tree.

use crate::formula::EvalCtx;
use crate::parser::BulletMLParser;
use crate::state::BulletMLState;
use crate::tree::{Name, NodeId, Type};
use crate::AppRunner;
use std::rc::Rc;

// Linear interpolation of y over the integer turn x (C++ `LinearFunc<int,double>`).
struct LinearFunc {
    first_x: f64,
    last_x: f64,
    first_y: f64,
    gradient: f64,
    last_y: f64,
}

impl LinearFunc {
    fn new(first_x: f64, last_x: f64, first_y: f64, last_y: f64) -> Self {
        LinearFunc {
            first_x,
            last_x,
            first_y,
            gradient: (last_y - first_y) / (last_x - first_x),
            last_y,
        }
    }
    fn get_value(&self, x: i32) -> f64 {
        self.first_y + self.gradient * (x as f64 - self.first_x)
    }
    fn is_last(&self, x: i32) -> bool {
        x as f64 >= self.last_x
    }
    fn get_last(&self) -> f64 {
        self.last_y
    }
}

struct RepeatElem {
    ite: i32,
    end: i32,
    act: NodeId,
}

// Per-top-action sub-runner (C++ `BulletMLRunnerImpl`).
struct Impl {
    parser: *const BulletMLParser,
    // The current resume node for each root (mutated as a turn is interrupted).
    node: Vec<NodeId>,
    // Snapshot of the original root nodes. The C++ engine nulls these nodes'
    // parent pointers; we instead treat a root's parent/next-sibling as absent,
    // which terminates the run_sub climb without mutating the shared tree.
    roots: Vec<NodeId>,
    act: Option<NodeId>,
    act_turn: i32,
    end_turn: i32,
    act_ite: usize,
    end: bool,
    params: Option<Rc<Vec<f64>>>,
    repeat_stack: Vec<RepeatElem>,
    ref_stack: Vec<(NodeId, Option<Rc<Vec<f64>>>)>,
    change_dir: Option<LinearFunc>,
    change_speed: Option<LinearFunc>,
    accelx: Option<LinearFunc>,
    accely: Option<LinearFunc>,
    // `Validatable<double>` -> Option (Some == validated).
    spd: Option<f64>,
    dir: Option<f64>,
    prev_spd: Option<f64>,
    prev_dir: Option<f64>,
}

impl Impl {
    fn new(parser: *const BulletMLParser, nodes: Vec<NodeId>, params: Option<Rc<Vec<f64>>>) -> Self {
        let act = nodes.first().copied();
        Impl {
            parser,
            roots: nodes.clone(),
            node: nodes,
            act,
            act_turn: -1,
            end_turn: 0,
            act_ite: 0,
            end: act.is_none(),
            params,
            repeat_stack: Vec::new(),
            ref_stack: Vec::new(),
            change_dir: None,
            change_speed: None,
            accelx: None,
            accely: None,
            spd: None,
            dir: None,
            prev_spd: None,
            prev_dir: None,
        }
    }

    fn parser(&self) -> &BulletMLParser {
        // SAFETY: the parser outlives this runner (see module docs).
        unsafe { &*self.parser }
    }

    fn name_of(&self, id: NodeId) -> Name {
        self.parser().node(id).name
    }

    // Parent of `id`, treating the runner's root nodes as parentless.
    fn parent_of(&self, id: NodeId) -> Option<NodeId> {
        if self.roots.contains(&id) {
            None
        } else {
            self.parser().node(id).parent
        }
    }

    // Next sibling of `id` (C++ `BulletMLNode::next`), root-aware.
    fn next_of(&self, id: NodeId) -> Option<NodeId> {
        if self.roots.contains(&id) {
            None
        } else {
            self.parser().next_sibling(id)
        }
    }

    fn is_end(&self) -> bool {
        self.end
    }

    fn is_turn_end(&self) -> bool {
        self.end || self.act_turn > self.end_turn
    }

    fn do_wait(&mut self, frame: i32) {
        if frame <= 0 {
            return;
        }
        self.act_turn += frame;
    }

    // Evaluate a node's formula against the current rank/params (getNumberContents).
    fn number_contents(&self, node: NodeId, app: &mut dyn AppRunner) -> f64 {
        let rank = app.get_rank();
        let params: &[f64] = self.params.as_deref().map(|v| v.as_slice()).unwrap_or(&[]);
        match self.parser().node(node).value.as_ref() {
            Some(e) => {
                let mut ctx = EvalCtx { rank, params, app };
                e.value(&mut ctx)
            }
            None => 0.0,
        }
    }

    // ---- direction / speed resolution (getDirection / getSpeed) ----

    fn get_direction(&mut self, dir_node: NodeId, prev_change: bool, app: &mut dyn AppRunner) -> f64 {
        let mut is_default = true;
        let mut dir = self.number_contents(dir_node, app);
        let ty = self.parser().node(dir_node).kind;
        if ty != Type::None {
            is_default = false;
            match ty {
                Type::Absolute => {
                    if self.parser().is_horizontal() {
                        dir -= 90.0;
                    }
                }
                Type::Relative => {
                    dir += app.get_bullet_direction();
                }
                Type::Sequence => match self.prev_dir {
                    None => {
                        dir = 0.0;
                        is_default = true;
                    }
                    Some(p) => dir += p,
                },
                // `aim` (and anything else) falls back to the aim direction.
                _ => is_default = true,
            }
        }
        if is_default {
            dir += app.get_aim_direction();
        }
        while dir > 360.0 {
            dir -= 360.0;
        }
        while dir < 0.0 {
            dir += 360.0;
        }
        if prev_change {
            self.prev_dir = Some(dir);
        }
        dir
    }

    fn get_speed(&mut self, spd_node: NodeId, app: &mut dyn AppRunner) -> f64 {
        let mut spd = self.number_contents(spd_node, app);
        let ty = self.parser().node(spd_node).kind;
        if ty != Type::None {
            if ty == Type::Relative {
                spd += app.get_bullet_speed();
            } else if ty == Type::Sequence {
                match self.prev_spd {
                    None => spd = 1.0,
                    Some(p) => spd += p,
                }
            }
        }
        self.prev_spd = Some(spd);
        spd
    }

    fn set_speed(&mut self, app: &mut dyn AppRunner) {
        if let Some(node) = self.parser().get_child(self.act.unwrap(), Name::Speed) {
            self.spd = Some(self.get_speed(node, app));
        }
    }

    fn set_direction(&mut self, app: &mut dyn AppRunner) {
        if let Some(node) = self.parser().get_child(self.act.unwrap(), Name::Direction) {
            self.dir = Some(self.get_direction(node, true, app));
        }
    }

    fn shot_init(&mut self) {
        self.spd = None;
        self.dir = None;
    }

    // ---- the frame step ----

    fn run(&mut self, app: &mut dyn AppRunner) {
        if self.is_end() {
            return;
        }

        self.changes(app);

        self.end_turn = app.get_turn();

        // Only waiting out the last wait / change-series.
        if self.act.is_none() {
            if !self.is_turn_end()
                && self.change_dir.is_none()
                && self.change_speed.is_none()
                && self.accelx.is_none()
                && self.accely.is_none()
            {
                self.end = true;
            }
            return;
        }

        self.act = Some(self.node[self.act_ite]);
        if self.act_turn == -1 {
            self.act_turn = app.get_turn();
        }

        self.run_sub(app);

        match self.act {
            None => {
                self.act_ite += 1;
                if self.node.len() != self.act_ite {
                    self.act = Some(self.node[self.act_ite]);
                }
            }
            Some(a) => self.node[self.act_ite] = a,
        }
    }

    fn changes(&mut self, app: &mut dyn AppRunner) {
        let now = app.get_turn();

        if self.change_dir.is_some() {
            let f = self.change_dir.as_ref().unwrap();
            let last = f.is_last(now);
            let v = if last { f.get_last() } else { f.get_value(now) };
            if last {
                self.change_dir = None;
            }
            app.do_change_direction(v);
        }
        if self.change_speed.is_some() {
            let f = self.change_speed.as_ref().unwrap();
            let last = f.is_last(now);
            let v = if last { f.get_last() } else { f.get_value(now) };
            if last {
                self.change_speed = None;
            }
            app.do_change_speed(v);
        }
        if self.accelx.is_some() {
            let f = self.accelx.as_ref().unwrap();
            let last = f.is_last(now);
            let v = if last { f.get_last() } else { f.get_value(now) };
            if last {
                self.accelx = None;
            }
            app.do_accel_x(v);
        }
        if self.accely.is_some() {
            let f = self.accely.as_ref().unwrap();
            let last = f.is_last(now);
            let v = if last { f.get_last() } else { f.get_value(now) };
            if last {
                self.accely = None;
            }
            app.do_accel_y(v);
        }
    }

    fn run_sub(&mut self, app: &mut dyn AppRunner) {
        while let Some(mut prev) = self.act {
            if self.is_turn_end() {
                break;
            }

            self.dispatch(prev, app);

            // Returning from a *Ref (its target is a direct child of <bulletml>).
            if self.act.is_none() {
                if let Some(pp) = self.parent_of(prev) {
                    if self.name_of(pp) == Name::Bulletml {
                        let (rprev, rpar) = self.ref_stack.pop().expect("ref stack underflow");
                        prev = rprev;
                        self.params = rpar;
                    }
                }
            }

            // Find the next node.
            if self.act.is_none() {
                self.act = self.next_of(prev);
            }

            // Climb up until we find a node to run (or run out).
            while self.act.is_none() {
                if let Some(pp) = self.parent_of(prev) {
                    if self.name_of(pp) == Name::Repeat {
                        let rep = self.repeat_stack.last_mut().expect("repeat stack underflow");
                        rep.ite += 1;
                        if rep.ite < rep.end {
                            self.act = Some(rep.act);
                            break;
                        } else {
                            self.repeat_stack.pop();
                        }
                    }
                }

                self.act = self.parent_of(prev);
                if self.act.is_none() {
                    break;
                }
                prev = self.act.unwrap();

                if let Some(pp) = self.parent_of(prev) {
                    if self.name_of(pp) == Name::Bulletml {
                        let (rprev, rpar) = self.ref_stack.pop().expect("ref stack underflow");
                        prev = rprev;
                        self.params = rpar;
                    }
                }

                self.act = self.next_of(prev);
            }
        }
    }

    fn dispatch(&mut self, node: NodeId, app: &mut dyn AppRunner) {
        match self.name_of(node) {
            Name::Bullet => self.run_bullet(app),
            Name::Action => self.run_action(),
            Name::Fire => self.run_fire(app),
            Name::ChangeDirection => self.run_change_direction(app),
            Name::ChangeSpeed => self.run_change_speed(app),
            Name::Accel => self.run_accel(app),
            Name::Wait => self.run_wait(app),
            Name::Repeat => self.run_repeat(app),
            Name::BulletRef => self.run_bullet_ref(app),
            Name::ActionRef => self.run_action_ref(app),
            Name::FireRef => self.run_fire_ref(app),
            Name::Vanish => self.run_vanish(app),
            // Container/value tags are never dispatched as commands.
            _ => self.act = None,
        }
    }

    fn run_bullet(&mut self, app: &mut dyn AppRunner) {
        self.set_speed(app);
        self.set_direction(app);
        if self.spd.is_none() {
            let v = app.get_default_speed();
            self.spd = Some(v);
            self.prev_spd = Some(v);
        }
        if self.dir.is_none() {
            let v = app.get_aim_direction();
            self.dir = Some(v);
            self.prev_dir = Some(v);
        }

        let act = self.act.unwrap();
        let has_action = self.parser().get_child(act, Name::Action).is_some()
            || self.parser().get_child(act, Name::ActionRef).is_some();

        if !has_action {
            app.create_simple_bullet(self.dir.unwrap(), self.spd.unwrap());
        } else {
            let mut acts = Vec::new();
            self.parser().get_all_children(act, Name::Action, &mut acts);
            self.parser().get_all_children(act, Name::ActionRef, &mut acts);
            let state = BulletMLState {
                parser: self.parser,
                nodes: acts,
                params: self.params.clone(),
            };
            app.create_bullet(state, self.dir.unwrap(), self.spd.unwrap());
        }

        self.act = None;
    }

    fn run_fire(&mut self, app: &mut dyn AppRunner) {
        self.shot_init();
        self.set_speed(app);
        self.set_direction(app);

        let act = self.act.unwrap();
        let bullet = self
            .parser()
            .get_child(act, Name::Bullet)
            .or_else(|| self.parser().get_child(act, Name::BulletRef))
            .expect("<fire> must contain <bullet> or <bulletRef>");
        self.act = Some(bullet);
    }

    fn run_action(&mut self) {
        let act = self.act.unwrap();
        if self.parser().child_count(act) == 0 {
            self.act = None;
        } else {
            self.act = self.parser().first_child(act);
        }
    }

    fn run_wait(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let frame = self.number_contents(act, app) as i32;
        self.do_wait(frame);
        self.act = None;
    }

    fn run_repeat(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let times = match self.parser().get_child(act, Name::Times) {
            Some(t) => t,
            None => return,
        };
        let times_num = self.number_contents(times, app) as i32;

        let action = self
            .parser()
            .get_child(act, Name::Action)
            .or_else(|| self.parser().get_child(act, Name::ActionRef))
            .expect("<repeat> must contain <action> or <actionRef>");

        self.repeat_stack.push(RepeatElem {
            ite: 0,
            end: times_num,
            act: action,
        });
        self.act = Some(action);
    }

    fn run_fire_ref(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let prev_para = self.params.clone();
        self.params = self.get_parameters(act, app);
        self.ref_stack.push((act, prev_para));
        let ref_id = self.parser().node(act).ref_id;
        self.act = Some(self.parser().get_fire_ref(ref_id));
    }

    fn run_action_ref(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let prev_para = self.params.clone();
        self.params = self.get_parameters(act, app);
        self.ref_stack.push((act, prev_para));
        let ref_id = self.parser().node(act).ref_id;
        self.act = Some(self.parser().get_action_ref(ref_id));
    }

    fn run_bullet_ref(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let prev_para = self.params.clone();
        self.params = self.get_parameters(act, app);
        self.ref_stack.push((act, prev_para));
        let ref_id = self.parser().node(act).ref_id;
        self.act = Some(self.parser().get_bullet_ref(ref_id));
    }

    fn run_change_direction(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let term_node = self.parser().get_child(act, Name::Term).unwrap();
        let term = self.number_contents(term_node, app) as i32;
        let dir_node = self.parser().get_child(act, Name::Direction).unwrap();
        let ty = self.parser().node(dir_node).kind;

        let dir = if ty != Type::Sequence {
            self.get_direction(dir_node, false, app)
        } else {
            self.number_contents(dir_node, app)
        };

        self.calc_change_direction(dir, term, ty == Type::Sequence, app);
        self.act = None;
    }

    fn run_change_speed(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let term_node = self.parser().get_child(act, Name::Term).unwrap();
        let term = self.number_contents(term_node, app) as i32;
        let spd_node = self.parser().get_child(act, Name::Speed).unwrap();
        let ty = self.parser().node(spd_node).kind;

        let spd = if ty != Type::Sequence {
            self.get_speed(spd_node, app)
        } else {
            self.number_contents(spd_node, app) * term as f64 + app.get_bullet_speed()
        };

        self.calc_change_speed(spd, term, app);
        self.act = None;
    }

    fn run_accel(&mut self, app: &mut dyn AppRunner) {
        let act = self.act.unwrap();
        let term_node = self.parser().get_child(act, Name::Term).unwrap();
        let term = self.number_contents(term_node, app) as i32;
        let hnode = self.parser().get_child(act, Name::Horizontal);
        let vnode = self.parser().get_child(act, Name::Vertical);

        if self.parser().is_horizontal() {
            if let Some(v) = vnode {
                let val = self.number_contents(v, app);
                let ty = self.parser().node(v).kind;
                self.calc_accel_x(val, term, ty, app);
            }
            if let Some(h) = hnode {
                let val = self.number_contents(h, app);
                let ty = self.parser().node(h).kind;
                self.calc_accel_y(-val, term, ty, app);
            }
        } else {
            if let Some(h) = hnode {
                let val = self.number_contents(h, app);
                let ty = self.parser().node(h).kind;
                self.calc_accel_x(val, term, ty, app);
            }
            if let Some(v) = vnode {
                let val = self.number_contents(v, app);
                let ty = self.parser().node(v).kind;
                self.calc_accel_y(val, term, ty, app);
            }
        }

        self.act = None;
    }

    fn run_vanish(&mut self, app: &mut dyn AppRunner) {
        app.do_vanish();
        self.act = None;
    }

    fn calc_change_direction(&mut self, direction: f64, term: i32, seq: bool, app: &mut dyn AppRunner) {
        let final_turn = (self.act_turn + term) as f64;
        let first = self.act_turn as f64;
        let dir_first = app.get_bullet_direction();

        if seq {
            self.change_dir = Some(LinearFunc::new(
                first,
                final_turn,
                dir_first,
                dir_first + direction * term as f64,
            ));
        } else {
            // Rotate the short way around.
            let dir_space1 = direction - dir_first;
            let dir_space2 = if dir_space1 > 0.0 {
                dir_space1 - 360.0
            } else {
                dir_space1 + 360.0
            };
            let dir_space = if dir_space1.abs() < dir_space2.abs() {
                dir_space1
            } else {
                dir_space2
            };
            self.change_dir = Some(LinearFunc::new(first, final_turn, dir_first, dir_first + dir_space));
        }
    }

    fn calc_change_speed(&mut self, speed: f64, term: i32, app: &mut dyn AppRunner) {
        let final_turn = (self.act_turn + term) as f64;
        let first = self.act_turn as f64;
        let spd_first = app.get_bullet_speed();
        self.change_speed = Some(LinearFunc::new(first, final_turn, spd_first, speed));
    }

    // calcAccelX in C++ takes the vertical-axis value and uses getBulletSpeedX.
    fn calc_accel_x(&mut self, value: f64, term: i32, ty: Type, app: &mut dyn AppRunner) {
        let final_turn = (self.act_turn + term) as f64;
        let first = self.act_turn as f64;
        let first_spd = app.get_bullet_speed_x();
        let final_spd = match ty {
            Type::Sequence => first_spd + value * term as f64,
            Type::Relative => first_spd + value,
            _ => value,
        };
        self.accelx = Some(LinearFunc::new(first, final_turn, first_spd, final_spd));
    }

    // calcAccelY in C++ takes the horizontal-axis value and uses getBulletSpeedY.
    fn calc_accel_y(&mut self, value: f64, term: i32, ty: Type, app: &mut dyn AppRunner) {
        let final_turn = (self.act_turn + term) as f64;
        let first = self.act_turn as f64;
        let first_spd = app.get_bullet_speed_y();
        let final_spd = match ty {
            Type::Sequence => first_spd + value * term as f64,
            Type::Relative => first_spd + value,
            _ => value,
        };
        self.accely = Some(LinearFunc::new(first, final_turn, first_spd, final_spd));
    }

    // Collect the <param> children of `node` into a 1-based parameter vector
    // (index 0 is an unused placeholder, as in C++). None if there are no params.
    fn get_parameters(&self, node: NodeId, app: &mut dyn AppRunner) -> Option<Rc<Vec<f64>>> {
        let children = self.parser().node(node).children.clone();
        let mut para: Option<Vec<f64>> = None;
        for c in children {
            if self.parser().node(c).name != Name::Param {
                continue;
            }
            let v = self.number_contents(c, app);
            para.get_or_insert_with(|| vec![0.0]).push(v);
        }
        para.map(Rc::new)
    }
}

/// Runs a BulletML pattern for a single bullet. Create one with
/// [`from_parser`](BulletMLRunner::from_parser) (a fresh top-level pattern) or
/// [`from_state`](BulletMLRunner::from_state) (a child spawned via
/// [`AppRunner::create_bullet`]), then call [`run`](BulletMLRunner::run) once per
/// frame until [`is_end`](BulletMLRunner::is_end) reports completion.
pub struct BulletMLRunner {
    impls: Vec<Impl>,
}

impl BulletMLRunner {
    /// Start a fresh run of `parser`'s top-level actions (those labelled `top*`).
    ///
    /// The parser must outlive the returned runner.
    pub fn from_parser(parser: &BulletMLParser) -> Self {
        let ptr = parser as *const BulletMLParser;
        let impls = parser
            .top_actions()
            .iter()
            .map(|&top| Impl::new(ptr, vec![top], None))
            .collect();
        BulletMLRunner { impls }
    }

    /// Resume the actions carried by a [`BulletMLState`] handed to the host by
    /// [`AppRunner::create_bullet`]. The originating parser must outlive the runner.
    pub fn from_state(state: BulletMLState) -> Self {
        let impl_ = Impl::new(state.parser, state.nodes, state.params);
        BulletMLRunner { impls: vec![impl_] }
    }

    /// Advance every sub-runner by one frame.
    pub fn run(&mut self, app: &mut dyn AppRunner) {
        for im in &mut self.impls {
            im.run(app);
        }
    }

    /// Whether the pattern has finished. Matches the C++ semantics: true as soon
    /// as **any** sub-runner has ended.
    pub fn is_end(&self) -> bool {
        self.impls.iter().any(|im| im.is_end())
    }
}
