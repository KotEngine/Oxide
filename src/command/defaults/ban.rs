use crate::command::{Command, CommandSender};
use crate::server::OxideServer;

pub struct BanCommand;
impl Command for BanCommand {
    fn name(&self) -> &str { "ban" }
    fn description(&self) -> &str { "Prevents a player from joining the server" }
    fn usage(&self) -> &str { "/ban <player> [reason]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.ban") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /ban <player> [reason]");
            return;
        }
        let target = &args[0];
        let reason = if args.len() > 1 { args[1..].join(" ") } else { "Banned by admin".to_string() };
        server.bans.write().insert(target.to_lowercase(), reason.clone());

        if let Some((addr, actual_name)) = server.find_player_by_name(target) {
            server.kick_player(addr, &format!("Banned: {}", reason));
            server.broadcast_message(&format!("Banned player {} for: {}", actual_name, reason));
        } else {
            sender.send_message(&format!("Banned player {}", target));
        }
    }
}

pub struct BanIpCommand;
impl Command for BanIpCommand {
    fn name(&self) -> &str { "ban-ip" }
    fn aliases(&self) -> Vec<&str> { vec!["banip"] }
    fn description(&self) -> &str { "Prevents an IP address from joining the server" }
    fn usage(&self) -> &str { "/ban-ip <ip|player> [reason]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.ban.ip") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /ban-ip <ip|player> [reason]");
            return;
        }
        let target = &args[0];
        let reason = if args.len() > 1 { args[1..].join(" ") } else { "IP Banned by admin".to_string() };

        let ip_to_ban = if let Some((addr, _)) = server.find_player_by_name(target) {
            addr.ip().to_string()
        } else {
            target.to_string()
        };

        server.banned_ips.write().insert(ip_to_ban.clone(), reason.clone());

        // Kick any connected player with that IP
        let matching_addrs: Vec<std::net::SocketAddr> = {
            let p = server.players.read();
            p.keys().filter(|a| a.ip().to_string() == ip_to_ban).copied().collect()
        };
        for addr in matching_addrs {
            server.kick_player(addr, &format!("IP Banned: {}", reason));
        }

        sender.send_message(&format!("Banned IP address {}", ip_to_ban));
    }
}

pub struct PardonCommand;
impl Command for PardonCommand {
    fn name(&self) -> &str { "pardon" }
    fn aliases(&self) -> Vec<&str> { vec!["unban"] }
    fn description(&self) -> &str { "Allows a banned player to join again" }
    fn usage(&self) -> &str { "/pardon <player>" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.pardon") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /pardon <player>");
            return;
        }
        let target = args[0].to_lowercase();
        if server.bans.write().remove(&target).is_some() {
            sender.send_message(&format!("Unbanned player {}", args[0]));
        } else {
            sender.send_message(&format!("Player '{}' is not banned", args[0]));
        }
    }
}

pub struct PardonIpCommand;
impl Command for PardonIpCommand {
    fn name(&self) -> &str { "pardon-ip" }
    fn aliases(&self) -> Vec<&str> { vec!["unbanip", "pardonip"] }
    fn description(&self) -> &str { "Allows a banned IP to join again" }
    fn usage(&self) -> &str { "/pardon-ip <ip>" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.pardon.ip") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /pardon-ip <ip>");
            return;
        }
        let ip = &args[0];
        if server.banned_ips.write().remove(ip).is_some() {
            sender.send_message(&format!("Unbanned IP address {}", ip));
        } else {
            sender.send_message(&format!("IP '{}' is not banned", ip));
        }
    }
}

pub struct BanListCommand;
impl Command for BanListCommand {
    fn name(&self) -> &str { "banlist" }
    fn description(&self) -> &str { "Views all players banned from this server" }
    fn usage(&self) -> &str { "/banlist [ips|players]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.banlist") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        let is_ips = args.first().map(|s| s.eq_ignore_ascii_case("ips")).unwrap_or(false);
        if is_ips {
            let guard = server.banned_ips.read();
            sender.send_message(&format!("There are {} total banned IP addresses:", guard.len()));
            for (ip, reason) in guard.iter() {
                sender.send_message(&format!("- {}: {}", ip, reason));
            }
        } else {
            let guard = server.bans.read();
            sender.send_message(&format!("There are {} total banned players:", guard.len()));
            for (name, reason) in guard.iter() {
                sender.send_message(&format!("- {}: {}", name, reason));
            }
        }
    }
}
