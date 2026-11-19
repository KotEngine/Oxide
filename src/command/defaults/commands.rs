use crate::command::{Command, CommandSender};
use crate::protocol::packet::DataPacket;
use crate::server::OxideServer;

macro_rules! simple_command {
    ($name:ident, $cmd:expr, $desc:expr, $perm:expr, $msg_fn:expr) => {
        pub struct $name;
        impl Command for $name {
            fn name(&self) -> &str { $cmd }
            fn description(&self) -> &str { $desc }
            fn permission(&self) -> Option<&str> { Some($perm) }
            fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
                ($msg_fn)(sender, server, args);
            }
        }
    };
}

simple_command!(KickCommand, "kick", "Kicks a player from the server", "oxide.command.kick", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    if args.is_empty() {
        sender.send_message("Usage: /kick <player> [reason]");
        return;
    }
    let target = &args[0];
    let reason = if args.len() > 1 { args[1..].join(" ") } else { "Kicked by admin".to_string() };
    if let Some((addr, actual_name)) = server.find_player_by_name(target) {
        server.kick_player(addr, &reason);
        server.broadcast_message(&format!("Kicked {} for: {}", actual_name, reason));
    } else {
        sender.send_message(&format!("Player '{}' not found", target));
    }
});

simple_command!(KillCommand, "kill", "Kills a player", "oxide.command.kill", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    if args.is_empty() {
        if let Some(addr) = sender.player_addr() {
            server.kill_player(addr);
        } else {
            sender.send_message("Usage: /kill <player>");
        }
    } else {
        let target = &args[0];
        if let Some((addr, actual_name)) = server.find_player_by_name(target) {
            server.kill_player(addr);
            sender.send_message(&format!("Killed {}", actual_name));
        } else {
            sender.send_message(&format!("Player '{}' not found", target));
        }
    }
});

simple_command!(SayCommand, "say", "Broadcasts a message to all players", "oxide.command.say", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    if args.is_empty() { sender.send_message("Usage: /say <message>"); return; }
    let msg = format!("[Server] {}", args.join(" "));
    server.broadcast_message(&msg);
});

simple_command!(MeCommand, "me", "Sends an action message", "oxide.command.me", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    if args.is_empty() { sender.send_message("Usage: /me <action>"); return; }
    let msg = format!("* {} {}", sender.name(), args.join(" "));
    server.broadcast_message(&msg);
});

simple_command!(TellCommand, "tell", "Sends a private message to a player", "oxide.command.tell", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    if args.len() < 2 { sender.send_message("Usage: /tell <player> <message>"); return; }
    let target = &args[0];
    let msg = args[1..].join(" ");
    if let Some((addr, actual_name)) = server.find_player_by_name(target) {
        server.send_message_to(addr, &format!("{} whispers to you: {}", sender.name(), msg));
        sender.send_message(&format!("You whisper to {}: {}", actual_name, msg));
    } else {
        sender.send_message(&format!("Player '{}' not found", target));
    }
});

simple_command!(SeedCommand, "seed", "Shows the world seed", "oxide.command.seed", |sender: &mut dyn CommandSender, _server: &OxideServer, _args: Vec<String>| {
    sender.send_message("Seed: 0");
});

simple_command!(StatusCommand, "status", "Shows server status and performance", "oxide.command.status", |sender: &mut dyn CommandSender, server: &OxideServer, _args: Vec<String>| {
    let count = server.players.read().values().filter(|s| s.logged_in).count();
    let max = server.config.read().get_int("max_players");
    sender.send_message("--- Server status ---");
    sender.send_message(&format!("Players: {}/{}", count, max));
    sender.send_message("TPS: 20.0 (interval 50ms)");
});

simple_command!(VersionCommand, "version", "Shows server version", "oxide.command.version", |sender: &mut dyn CommandSender, _server: &OxideServer, _args: Vec<String>| {
    sender.send_message("This server is running Oxide v0.1.0 (MCPE 1.1.5 / Protocol 113)");
});

simple_command!(DifficultyCommand, "difficulty", "Sets game difficulty", "oxide.command.difficulty", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    if args.is_empty() { sender.send_message("Usage: /difficulty <peaceful|easy|normal|hard>"); return; }
    let diff_names = ["peaceful", "easy", "normal", "hard"];
    let level = args[0].parse::<u32>().ok()
        .filter(|&n| n < 4)
        .or_else(|| diff_names.iter().position(|n| n.starts_with(&args[0].to_lowercase())).map(|i| i as u32));
    match level {
        Some(l) => {
            let pk = crate::protocol::packets::SetDifficultyPacket::new(l);
            server.broadcast_packet(pk.encode());
            sender.send_message(&format!("Set game difficulty to {}", diff_names[l as usize]));
        }
        None => sender.send_message(&format!("Unknown difficulty: {}", args[0])),
    }
});

simple_command!(SpawnpointCommand, "spawnpoint", "Sets a player's spawn point", "oxide.command.spawnpoint", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    let target_addr = if args.is_empty() {
        sender.player_addr()
    } else {
        server.find_player_by_name(&args[0]).map(|(a, _)| a)
    };
    if let Some(addr) = target_addr {
        let coords = {
            let p = server.players.read();
            p.get(&addr).map(|s| (s.x as i32, s.y as u32, s.z as i32))
        };
        if let Some((x, y, z)) = coords {
            let pk = crate::protocol::packets::SetSpawnPositionPacket::new_player(x, y as i32, z);
            server.send_packet_to(addr, pk.encode());
            sender.send_message(&format!("Set spawn point to {}, {}, {}", x, y, z));
        }
    } else {
        sender.send_message("Usage: /spawnpoint [player]");
    }
});

simple_command!(SetworldspawnCommand, "setworldspawn", "Sets the world spawn coordinates", "oxide.command.setworldspawn", |sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>| {
    let (x, y, z) = if args.len() >= 3 {
        (args[0].parse().unwrap_or(0), args[1].parse().unwrap_or(4), args[2].parse().unwrap_or(0))
    } else if let Some(addr) = sender.player_addr() {
        let p = server.players.read();
        p.get(&addr).map(|s| (s.x as i32, s.y as i32, s.z as i32)).unwrap_or((0, 4, 0))
    } else {
        (0, 4, 0)
    };
    {
        let mut w = server.world.write();
        w.spawn_x = x;
        w.spawn_y = y;
        w.spawn_z = z;
    }
    let pk = crate::protocol::packets::SetSpawnPositionPacket::new_world(x, y, z);
    server.broadcast_packet(pk.encode());
    sender.send_message(&format!("Set world spawn point to {}, {}, {}", x, y, z));
});

simple_command!(HelpCommand, "help", "Shows available commands", "oxide.command.help", |sender: &mut dyn CommandSender, server: &OxideServer, _args: Vec<String>| {
    sender.send_message("--- Available Commands ---");
    let cmds = server.command_manager.all_commands();
    for c in cmds {
        sender.send_message(&format!("/{} - {}", c.name(), c.description()));
    }
});
