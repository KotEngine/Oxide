use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::{MobEffectPacket, entity_packets};
use crate::protocol::packet::DataPacket;

pub struct EffectCommand;

const EFFECTS: &[(&str, i32)] = &[
    ("speed", 1),
    ("slowness", 2),
    ("haste", 3),
    ("mining_fatigue", 4),
    ("strength", 5),
    ("instant_health", 6),
    ("instant_damage", 7),
    ("jump_boost", 8),
    ("nausea", 9),
    ("regeneration", 10),
    ("resistance", 11),
    ("fire_resistance", 12),
    ("water_breathing", 13),
    ("invisibility", 14),
    ("blindness", 15),
    ("night_vision", 16),
    ("hunger", 17),
    ("weakness", 18),
    ("poison", 19),
    ("wither", 20),
    ("health_boost", 21),
    ("absorption", 22),
    ("saturation", 23),
    ("levitation", 24),
];

fn parse_effect(input: &str) -> Option<i32> {
    if let Ok(id) = input.parse::<i32>() {
        return Some(id);
    }
    let lower = input.to_lowercase();
    for (name, id) in EFFECTS {
        if *name == lower || name.starts_with(&lower) {
            return Some(*id);
        }
    }
    None
}

impl Command for EffectCommand {
    fn name(&self) -> &str { "effect" }
    fn description(&self) -> &str { "Gives or clears potion effects for a player" }
    fn usage(&self) -> &str { "/effect <player> <effect|clear> [seconds] [amplifier] [hideParticles]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.effect") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.len() < 2 {
            sender.send_message("Usage: /effect <player> <effect|clear> [seconds] [amplifier] [hideParticles]");
            return;
        }

        let target_name = &args[0];
        let target = server.find_player_by_name(target_name);
        let (addr, actual_name) = match target {
            Some(t) => t,
            None => {
                sender.send_message(&format!("Player '{}' not found", target_name));
                return;
            }
        };

        let eid = server.players.read().get(&addr).map(|s| s.entity_id).unwrap_or(0);

        if args[1].eq_ignore_ascii_case("clear") {
            // Remove all effects
            for (_, eff_id) in EFFECTS {
                let pk = MobEffectPacket {
                    entity_runtime_id: eid,
                    event_id: entity_packets::EVENT_REMOVE,
                    effect_id: *eff_id,
                    amplifier: 0,
                    particles: false,
                    duration: 0,
                };
                server.send_packet_to(addr, pk.encode());
            }
            sender.send_message(&format!("Cleared all effects for {}", actual_name));
            return;
        }

        let effect_id = match parse_effect(&args[1]) {
            Some(id) => id,
            None => {
                sender.send_message(&format!("Unknown effect: {}", args[1]));
                return;
            }
        };

        let seconds = args.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(30);
        let amplifier = args.get(3).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let hide_particles = args.get(4).map(|s| s == "true" || s == "1").unwrap_or(false);

        let duration_ticks = seconds * 20;

        let pk = MobEffectPacket {
            entity_runtime_id: eid,
            event_id: entity_packets::EVENT_ADD,
            effect_id,
            amplifier,
            particles: !hide_particles,
            duration: duration_ticks,
        };
        server.send_packet_to(addr, pk.encode());

        sender.send_message(&format!(
            "Applied effect {} (ID {}) * {} to {} for {} seconds",
            args[1], effect_id, amplifier, actual_name, seconds
        ));
    }
}
