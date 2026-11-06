//! Example mod (`mods: ["example"]` in the config, or `--mods example`). Shows every kind of registration:
//! * an engine hook: `global_stage` runs once per tick at the end of P5, with the world writable;
//! * an HTTP route: `GET /api/example/stats` reads the last published world;
//! * a socket channel: `subscribe example:ticks` gets the game time after every tick;
//! * `config.backend` assignments, as the official example mods make them (launcher/init_dist/example-mods:
//!   welcome-text.js, custom-objects/simple.js): the welcome text, and how the client shows a `myobject`;
//! * game-slot JS: the `MyObject` prototype players' scripts get for `myobject` objects, found with
//!   `room.find(10000)` (custom-objects/simple.js's `registerCustomObjectPrototype`).
//! Hooks are plain `fn`s: state a mod keeps between calls lives in statics.
//!
//! Try the custom object from the CLI: `storage.db['rooms.objects'].insert({type: 'myobject', room: 'W1N1', x: 20,
//! y: 20})`; the client draws a green circle there.

use backend::http::{Request, Response};
use backend::mods::{Manifest, Mod, Registry};
use backend::{ApiCtx, Route};
use engine::doc::{Atom, Val};
use engine::processor::global::GlobalCtx;
use engine::world::{UserIdx, World, WorldView};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use util::json::Json;

/// custom-objects/simple.js: `config.backend.renderer.metadata['myobject']`, a green circle.
const MYOBJECT_RENDERER: &str = r#"{"processors":[{"type":"draw","once":true,"payload":{"drawings":[
    {"method":"beginFill","params":[7864183]},{"method":"drawCircle","params":[0,0,40]},{"method":"endFill"}]}}]}"#;

/// custom-objects/simple.js: `config.engine.registerCustomObjectPrototype('myobject', 'MyObject', {findConstant:
/// 10000})`, through the game slot's exports.
const GAME_JS: &str = "module.exports.customObjectPrototypes = [['myobject', 'MyObject', {findConstant: 10000}]];";

pub struct Example;

/// What the server registers (modlist finds it by this name).
pub static MOD: Example = Example;

impl Mod for Example {
    fn manifest(&self) -> Manifest {
        Manifest { name: "example", version: env!("CARGO_PKG_VERSION"), dependencies: &[] }
    }

    fn register(&self, r: &mut Registry) {
        r.engine.processor.global_stage.push(count_tick);
        r.routes.push(Route { method: "GET", path: "/api/example/stats", handler: stats });
        r.socket_channels.push(("example:ticks", ticks_channel));
        r.server_data.push(("welcomeText", Json::Str("<h4>Welcome!</h4>This server runs the Krebs example mod.".into())));
        r.server_data.push((
            "customObjectTypes.myobject",
            Json::Obj(vec![("sidepanel".into(), Json::Str("<div><label>An example object</label></div>".into()))]),
        ));
        r.server_data.push(("renderer.metadata.myobject", Json::parse(MYOBJECT_RENDERER).expect("static json")));
        r.js_game.push(GAME_JS);
    }
}

/// `example:ticks`: accepted for anyone (the reply on subscribe is the current time), then the time every tick.
fn ticks_channel(_user: Option<UserIdx>, channel: &str, w: &WorldView) -> Option<Json> {
    (channel == "example:ticks").then(|| Json::Obj(vec![("time".into(), Json::Num(w.time as f64))]))
}

/// Ticks processed since the server started.
static TICKS: AtomicU64 = AtomicU64::new(0);

fn count_tick(_w: &mut World, _ctx: &GlobalCtx) {
    TICKS.fetch_add(1, Relaxed);
}

/// `{ok: 1, time, ticks, creeps}`: the game time, ticks counted by the hook, creeps in the world.
fn stats(_req: &Request, ctx: &ApiCtx) -> Response {
    let w = ctx.published.get();
    let creep = |d: &&engine::doc::DocRef| d.obj.get(Atom::intern("type")).and_then(Val::as_str) == Some("creep");
    let creeps = w.rooms.iter().map(|r| r.objects.iter().filter(creep).count()).sum::<usize>();
    Response::ok(Json::Obj(vec![
        ("time".into(), Json::Num(w.time as f64)),
        ("ticks".into(), Json::Num(TICKS.load(Relaxed) as f64)),
        ("creeps".into(), Json::Num(creeps as f64)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_its_pieces() {
        let mut r = Registry::default();
        MOD.register(&mut r);
        assert_eq!(MOD.manifest().name, "example");
        assert_eq!(r.engine.processor.global_stage.len(), 1);
        assert!(r.routes.iter().any(|x| x.method == "GET" && x.path == "/api/example/stats"));
        assert_eq!(r.socket_channels.len(), 1);
        assert_eq!(r.server_data.len(), 3);
        assert_eq!(r.js_game, [GAME_JS]);
    }
}
