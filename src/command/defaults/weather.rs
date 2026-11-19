use crate::command::{Command, CommandSender};
use crate::server::OxideServer;
use crate::level::WeatherState;

pub struct WeatherCommand;

impl Command for WeatherCommand {
    fn name(&self) -> &str { "weather" }
    fn description(&self) -> &str { "Sets the world weather" }
    fn usage(&self) -> &str { "/weather <clear|rain|thunder> [duration]" }
    fn permission(&self) -> Option<&str> { Some("oxide.command.weather") }

    fn execute(&self, sender: &mut dyn CommandSender, server: &OxideServer, args: Vec<String>) {
        if args.is_empty() {
            sender.send_message("Usage: /weather <clear|rain|thunder> [duration]");
            return;
        }

        let mode_str = args[0].to_lowercase();
        let (state, name) = match mode_str.as_str() {
            "clear" | "sun" => (WeatherState::Clear, "clear"),
            "rain" => (WeatherState::Rain, "rain"),
            "thunder" | "storm" => (WeatherState::Thunder, "thunder"),
            _ => {
                sender.send_message(&format!("Unknown weather type: {}", args[0]));
                return;
            }
        };

        let duration_seconds = args.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(600);
        let duration_ticks = duration_seconds.saturating_mul(20);

        let weather = server.weather.clone();
        let rak = server.rak.clone();
        let players = server.players.clone();
        tokio::spawn(async move {
            weather.set_weather(state, duration_ticks, &rak, &players).await;
        });

        sender.send_message(&format!("Set weather to {} for {} seconds", name, duration_seconds));
    }
}
