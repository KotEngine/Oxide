use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::protocol::packets::ContainerSetContentPacket;
use crate::protocol::packet::DataPacket;
use crate::protocol::types::{ContainerIds, Item};

pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &str { "clear" }
    fn description(&self) -> &str { "Clears items from player inventory" }
    fn usage(&self) -> &str { "/clear [player] [item_id] [meta] [count]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.clear") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        let target_res = if !args.is_empty() {
            server.find_player_by_name(&args[0])
        } else if let Some(addr) = sender.player_addr() {
            Some((addr, sender.name().to_string()))
        } else {
            None
        };

        let (addr, actual_name) = match target_res {
            Some(t) => t,
            None => {
                sender.send_message("Target player not found");
                return;
            }
        };

        let filter_id = args.get(1).and_then(|s| s.parse::<i32>().ok());
        let filter_meta = args.get(2).and_then(|s| s.parse::<i32>().ok());

        let (eid, cleared_count, new_items) = {
            let mut players = server.players.write();
            if let Some(state) = players.get_mut(&addr) {
                let mut count = 0;
                for i in 0..state.inventory.size() {
                    let it = state.inventory.get_item(i);
                    if it.id != 0 {
                        let match_id = filter_id.map(|id| id == it.id).unwrap_or(true);
                        let match_meta = filter_meta.map(|m| m == it.meta).unwrap_or(true);
                        if match_id && match_meta {
                            count += it.count;
                            state.inventory.set_item(i, Item::air());
                        }
                    }
                }
                let all_items = state.inventory.all_items();
                (state.entity_id, count, all_items)
            } else {
                (0, 0, Vec::new())
            }
        };

        let mut pk = ContainerSetContentPacket::new(ContainerIds::INVENTORY as u32);
        pk.target_eid = eid;
        pk.items = new_items;
        server.send_packet_to(addr, pk.encode());

        sender.send_message(&format!("Cleared {} items from {}", cleared_count, actual_name));
        if sender.name() != actual_name {
            server.send_message_to(addr, &format!("Your inventory was cleared ({} items removed)", cleared_count));
        }
    }
}
