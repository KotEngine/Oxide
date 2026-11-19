use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::ContainerSetSlotPacket;
use crate::protocol::packet::DataPacket;

pub struct EnchantCommand;

const ENCHANTMENTS: &[(&str, i32)] = &[
    ("protection", 0),
    ("fire_protection", 1),
    ("feather_falling", 2),
    ("blast_protection", 3),
    ("projectile_protection", 4),
    ("respiration", 5),
    ("aqua_affinity", 6),
    ("thorns", 7),
    ("depth_strider", 8),
    ("sharpness", 9),
    ("smite", 10),
    ("bane_of_arthropods", 11),
    ("knockback", 12),
    ("fire_aspect", 13),
    ("looting", 14),
    ("efficiency", 15),
    ("silk_touch", 16),
    ("unbreaking", 17),
    ("fortune", 18),
    ("power", 19),
    ("punch", 20),
    ("flame", 21),
    ("infinity", 22),
    ("luck_of_the_sea", 23),
    ("lure", 24),
    ("frost_walker", 25),
    ("mending", 26),
];

fn parse_enchantment(input: &str) -> Option<i32> {
    if let Ok(id) = input.parse::<i32>() {
        return Some(id);
    }
    let lower = input.to_lowercase();
    for (name, id) in ENCHANTMENTS {
        if *name == lower || name.starts_with(&lower) {
            return Some(*id);
        }
    }
    None
}

impl Command for EnchantCommand {
    fn name(&self) -> &str { "enchant" }
    fn description(&self) -> &str { "Enchants the held item of a player" }
    fn usage(&self) -> &str { "/enchant <player> <enchantment> [level]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.enchant") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.len() < 2 {
            sender.send_message("Usage: /enchant <player> <enchantment> [level]");
            return;
        }

        let target_name = &args[0];
        let target = match server.find_player_by_name(target_name) {
            Some(t) => t,
            None => {
                sender.send_message(&format!("Player '{}' not found", target_name));
                return;
            }
        };

        let ench_id = match parse_enchantment(&args[1]) {
            Some(id) => id,
            None => {
                sender.send_message(&format!("Unknown enchantment: {}", args[1]));
                return;
            }
        };

        let level = args.get(2).and_then(|s| s.parse::<i16>().ok()).unwrap_or(1);

        let (addr, actual_name) = target;
        let slot_info = {
            let mut players = server.players.write();
            if let Some(state) = players.get_mut(&addr) {
                let slot = state.inventory.held_index();
                let mut item = state.inventory.get_item(slot);
                if item.id == 0 {
                    None
                } else {
                    crate::item::enchantment::apply_enchantment_to_item(&mut item, ench_id as u16, level as u16);
                    state.inventory.set_item(slot, item.clone());
                    Some((slot, item))
                }
            } else {
                None
            }
        };

        match slot_info {
            Some((slot, item)) => {
                let pk = ContainerSetSlotPacket::new(0, slot as i32, item);
                server.send_packet_to(addr, pk.encode());
                sender.send_message(&format!("Enchanted held item of {} with ID {} level {}", actual_name, ench_id, level));
            }
            None => {
                sender.send_message(&format!("{} is not holding any item to enchant", actual_name));
            }
        }
    }
}
