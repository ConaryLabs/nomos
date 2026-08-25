//! Fixtures: the six committed areas, compiled in memory.
//!
//! The rendering plans are the committed `rendering-plan.example.json` files;
//! the executable semantics are obtained by compiling each area's `world.nomos`
//! through `nomos_compiler::compile_world_package` and taking the
//! `simulation.json` member. Nothing here reads `target/`, so the tests do not
//! depend on a prior `gaol capture`.
//!
//! The source path matters and is not decorative: it appears in every claim's
//! `SourceSpan`, so it is inside the projection's bytes and therefore inside
//! the digest the rendering plan published. Compiling with a different path
//! produces a projection the plan will refuse (`PL0502`), which is the point of
//! the check.

#![allow(dead_code)]

use nomos_core::SourcePath;
use nomos_play::{Direction, PlayCommand, PlaySession};

/// One committed area's bytes.
pub struct AreaBytes {
    pub id: &'static str,
    pub plan: &'static [u8],
    pub source: &'static str,
    pub source_path: &'static str,
}

macro_rules! area {
    ($id:literal) => {
        AreaBytes {
            id: $id,
            plan: include_bytes!(concat!(
                "../../../../experiments/executable-gaol/areas/",
                $id,
                "/rendering-plan.example.json"
            )),
            source: include_str!(concat!(
                "../../../../experiments/executable-gaol/areas/",
                $id,
                "/world.nomos"
            )),
            source_path: concat!("experiments/executable-gaol/areas/", $id, "/world.nomos"),
        }
    };
}

/// The six committed areas, in route order.
pub const ROUTE: [&str; 6] = [
    "cistern-walk",
    "ember-vault",
    "gloam-bastion",
    "drowned-stair",
    "ossuary-reach",
    "north-gaol",
];

/// Every committed area, by identifier.
#[must_use]
pub fn area(id: &str) -> AreaBytes {
    match id {
        "cistern-walk" => area!("cistern-walk"),
        "ember-vault" => area!("ember-vault"),
        "gloam-bastion" => area!("gloam-bastion"),
        "drowned-stair" => area!("drowned-stair"),
        "north-gaol" => area!("north-gaol"),
        "ossuary-reach" => area!("ossuary-reach"),
        other => panic!("no committed area `{other}`"),
    }
}

/// One area's simulation projection, compiled in memory.
#[must_use]
pub fn semantics(id: &str) -> Vec<u8> {
    let bytes = area(id);
    let world = nomos_compiler::compile_world_package(
        bytes.source,
        SourcePath::new(bytes.source_path).expect("the fixture path is repository-relative"),
    )
    .expect("the committed area compiles");
    world
        .members()
        .expect("the package names its members")
        .into_iter()
        .find(|(name, _)| name.as_str() == "simulation.json")
        .expect("the package declares a simulation member")
        .1
}

/// One area's committed rendering plan.
#[must_use]
pub fn plan(id: &str) -> Vec<u8> {
    area(id).plan.to_vec()
}

/// A session opened at one area.
#[must_use]
pub fn session_at(id: &str) -> PlaySession {
    PlaySession::start(&plan(id), &semantics(id)).expect("the committed area opens")
}

/// A session opened at the route's start area.
#[must_use]
pub fn session() -> PlaySession {
    session_at(ROUTE[0])
}

/// Enters the next area of the route.
pub fn enter(session: &mut PlaySession, id: &str) {
    session
        .enter(&plan(id), &semantics(id))
        .expect("the route continues here");
}

/// `move {direction}`.
#[must_use]
pub const fn step(direction: Direction) -> PlayCommand {
    PlayCommand::Move { direction }
}

/// The six key sequences the smoke lane's route solver produces today, one per
/// area, in route order. `^ v < >` are the four lattice directions and `*` is
/// the interaction the enumeration offers first at the cell the player is
/// standing on. Recorded here so a native test pins the numbers the browser
/// lane then has to agree with.
pub const ROUTE_KEYS: [&str; 6] = [
    "^^^<<<<<<^**>^",
    "<<<<<^^^>^^**>^",
    "^^^<<<v^^^**<^",
    "^<<>^^^**^^",
    "^^>^^>>>^**>^",
    "^^^^>>**>^",
];

/// The direction one route letter names, or `None` for the interaction key.
#[must_use]
pub const fn key(letter: char) -> Option<Direction> {
    match letter {
        '^' => Some(Direction::North),
        'v' => Some(Direction::South),
        '<' => Some(Direction::West),
        '>' => Some(Direction::East),
        _ => None,
    }
}

/// Drives one area's key sequence through a live session, sending the first
/// available interaction for each `*`.
///
/// # Panics
///
/// Panics if a `*` finds nothing available, which would mean the enumeration
/// and the route disagree — the thing `tests/corpus.rs` exists to catch.
pub fn drive(session: &mut PlaySession, keys: &str) {
    for letter in keys.chars() {
        let command = match key(letter) {
            Some(direction) => step(direction),
            None => {
                let available = nomos_play::batch::available_interactions(session.live())
                    .expect("the kernel resolves the available interactions");
                let (entity, action) = available
                    .first()
                    .expect("the route expects an interaction here")
                    .clone();
                PlayCommand::Interact { entity, action }
            }
        };
        session.step(&command).expect("the input is well formed");
    }
}

/// Plays the whole six-area route, entering each area as the crossing names it.
#[must_use]
pub fn play_route() -> PlaySession {
    let mut live = session();
    for (index, area_id) in ROUTE.iter().enumerate() {
        if index > 0 {
            enter(&mut live, area_id);
        }
        drive(&mut live, ROUTE_KEYS[index]);
    }
    live
}
