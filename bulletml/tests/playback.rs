//! Integration tests for the BulletML port: formula evaluation, parsing every
//! real game pattern, and a golden playback against a recording host.

use bulletml::{AppRunner, BulletMLParser, BulletMLRunner, BulletMLState};
use std::path::PathBuf;

// A host that records what the runner asks it to do, with fixed bullet state.
#[derive(Default)]
struct Recorder {
    turn: i32,
    rank: f64,
    aim: f64,
    simple: Vec<(i32, f64, f64)>, // (turn, direction, speed)
    states: usize,
    vanished: bool,
    dir: f64,
    speed: f64,
}

impl AppRunner for Recorder {
    fn get_bullet_direction(&mut self) -> f64 {
        self.dir
    }
    fn get_aim_direction(&mut self) -> f64 {
        self.aim
    }
    fn get_bullet_speed(&mut self) -> f64 {
        self.speed
    }
    fn get_rank(&mut self) -> f64 {
        self.rank
    }
    fn create_simple_bullet(&mut self, direction: f64, speed: f64) {
        self.simple.push((self.turn, direction, speed));
    }
    fn create_bullet(&mut self, _state: BulletMLState, _direction: f64, _speed: f64) {
        self.states += 1;
    }
    fn get_turn(&mut self) -> i32 {
        self.turn
    }
    fn do_vanish(&mut self) {
        self.vanished = true;
    }
    fn get_rand(&mut self) -> f64 {
        0.5
    }
}

fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/bulletdata")
}

#[test]
fn parses_every_game_pattern() {
    let root = assets_dir();
    let mut count = 0;
    for category in std::fs::read_dir(&root).expect("read bulletdata") {
        let dir = category.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("xml") {
                continue;
            }
            BulletMLParser::parse_file(&path)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
            count += 1;
        }
    }
    assert!(count > 0, "no BulletML patterns found under {}", root.display());
}

// The canonical small/shot.xml:
//   <action label="top">
//     <wait>60-$rank*60</wait>
//     <fire><speed>1</speed><bullet/></fire>
//     <wait>120-$rank*80</wait>
//   </action>
// With rank=0.5 the first wait is 60-30 = 30 frames, so the (aimed, speed-1)
// bullet must fire on turn 30 and not before.
#[test]
fn golden_shot_fires_after_wait() {
    let xml = r#"<?xml version="1.0" ?>
        <bulletml type="vertical" xmlns="http://www.asahi-net.or.jp/~cs8k-cyu/bulletml">
          <action label="top">
            <wait>60-$rank*60</wait>
            <fire><speed>1</speed><bullet/></fire>
            <wait>120-$rank*80</wait>
          </action>
        </bulletml>"#;
    let parser = BulletMLParser::parse_str(xml).unwrap();
    let mut runner = BulletMLRunner::from_parser(&parser);

    let mut host = Recorder {
        rank: 0.5,
        aim: 42.0,
        ..Default::default()
    };

    for turn in 0..40 {
        host.turn = turn;
        runner.run(&mut host);
    }

    assert_eq!(host.simple.len(), 1, "exactly one bullet should be fired");
    let (turn, dir, speed) = host.simple[0];
    assert_eq!(turn, 30, "bullet fires after the 30-frame wait");
    assert_eq!(speed, 1.0, "speed is 1");
    assert_eq!(dir, 42.0, "no <direction> means aim direction");
}

#[test]
fn parses_horizontal_orientation() {
    let xml = r#"<bulletml type="horizontal" xmlns="http://www.asahi-net.or.jp/~cs8k-cyu/bulletml">
          <action label="top"><fire><bullet/></fire></action>
        </bulletml>"#;
    let parser = BulletMLParser::parse_str(xml).unwrap();
    assert!(parser.is_horizontal());

    let vert = BulletMLParser::parse_str(
        r#"<bulletml type="vertical" xmlns="http://www.asahi-net.or.jp/~cs8k-cyu/bulletml">
             <action label="top"><fire><bullet/></fire></action>
           </bulletml>"#,
    )
    .unwrap();
    assert!(!vert.is_horizontal());
}

// A bulletRef with parameters, resolved by label, plus $1/$rank in a formula.
#[test]
fn resolves_refs_and_params() {
    let xml = r#"<bulletml xmlns="http://www.asahi-net.or.jp/~cs8k-cyu/bulletml">
          <action label="top">
            <fire>
              <bulletRef label="aimed"><param>3</param></bulletRef>
            </fire>
          </action>
          <bullet label="aimed">
            <speed>$1+$rank</speed>
          </bullet>
        </bulletml>"#;
    let parser = BulletMLParser::parse_str(xml).unwrap();
    let mut runner = BulletMLRunner::from_parser(&parser);
    let mut host = Recorder {
        rank: 0.25,
        ..Default::default()
    };
    for turn in 0..3 {
        host.turn = turn;
        runner.run(&mut host);
    }
    assert_eq!(host.simple.len(), 1);
    // speed = $1 + $rank = 3 + 0.25
    assert_eq!(host.simple[0].2, 3.25);
}
