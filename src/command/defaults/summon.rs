use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct SummonCommand;

const ENTITY_NAMES: &[(&str, i32, f32)] = &[
    ("chicken", 10, 4.0),
    ("cow", 11, 10.0),
    ("pig", 12, 10.0),
    ("sheep", 13, 8.0),
    ("wolf", 14, 8.0),
    ("villager", 15, 20.0),
    ("mooshroom", 16, 10.0),
    ("squid", 17, 10.0),
    ("rabbit", 18, 3.0),
    ("bat", 19, 6.0),
    ("iron_golem", 20, 100.0),
    ("snow_golem", 21, 4.0),
    ("ocelot", 22, 10.0),
    ("horse", 23, 20.0),
    ("zombie", 32, 20.0),
    ("creeper", 33, 20.0),
    ("skeleton", 34, 20.0),
    ("spider", 35, 16.0),
    ("pig_zombie", 36, 20.0),
    ("slime", 37, 16.0),
    ("enderman", 38, 40.0),
    ("silverfish", 40, 8.0),
    ("blaze", 41, 20.0),
    ("magma_cube", 42, 16.0),
    ("witch", 45, 26.0),
    ("stray", 46, 20.0),
    ("husk", 47, 20.0),
    ("wither_skeleton", 48, 20.0),
    ("tnt", 65, 1.0),
];

fn parse_coord(coord_str: &str, base: f64) -> Option<f64> {
    if let Some(rest) = coord_str.strip_prefix('~') {
        if rest.is_empty() {
            Some(base)
        } else {
            let offset: f64 = rest.parse().ok()?;
            Some(base + offset)
        }
    } else {
        coord_str.parse::<f64>().ok()
    }
}

impl Command for SummonCommand {
    fn name(&self) -> &str { "summon" }
    fn description(&self) -> &str { "Spawns an entity into the world" }
    fn usage(&self) -> &str { "/summon <entity> [x] [y] [z]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.summon") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /summon <entity> [x] [y] [z]");
            return;
        }

        let ent_name = args[0].to_lowercase();
        let (type_id, health, canonical_name) = if let Ok(id) = ent_name.parse::<i32>() {
            let name = ENTITY_NAMES.iter().find(|(_, i, _)| *i == id).map(|(n, _, _)| *n).unwrap_or("Entity");
            (id, 20.0f32, name)
        } else if let Some((name, id, hp)) = ENTITY_NAMES.iter().find(|(n, _, _)| *n == ent_name || n.starts_with(&ent_name)) {
            (*id, *hp, *name)
        } else {
            sender.send_message(&format!("Unknown entity type: {}", args[0]));
            return;
        };

        let (base_x, base_y, base_z) = if let Some(addr) = sender.player_addr() {
            server.players.read().get(&addr).map(|s| (s.x, s.y, s.z)).unwrap_or((0.0, 4.0, 0.0))
        } else {
            (0.0, 4.0, 0.0)
        };

        let x = if args.len() > 1 { parse_coord(&args[1], base_x).unwrap_or(base_x) } else { base_x };
        let y = if args.len() > 2 { parse_coord(&args[2], base_y).unwrap_or(base_y) } else { base_y };
        let z = if args.len() > 3 { parse_coord(&args[3], base_z).unwrap_or(base_z) } else { base_z };

        let eid = {
            let mut guard = server.next_entity_id.write();
            let id = *guard;
            *guard += 1;
            id
        };

        let manager = server.entity_manager.clone();
        let rak = server.rak.clone();
        let players = server.players.clone();

        if type_id == 65 {
            tokio::spawn(async move {
                manager.spawn_tnt(eid, x, y, z, 80, &rak, &players).await;
            });
        } else {
            tokio::spawn(async move {
                manager.spawn_mob(eid, type_id, canonical_name.to_string(), x, y, z, health, &rak, &players).await;
            });
        }

        sender.send_message(&format!("Summoned {} (ID {}) at {:.1}, {:.1}, {:.1}", canonical_name, type_id, x, y, z));
    }
}
