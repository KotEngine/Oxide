use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use crate::raknet::RakNetServer;
use crate::protocol::handler::PlayerState;
use crate::protocol::packets::{LevelEventPacket, level_event};
use crate::protocol::packet::DataPacket;
use crate::protocol::batch::BatchPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherState {
    Clear,
    Rain,
    Thunder,
}

pub struct WeatherSimulation {
    state: RwLock<WeatherState>,
    duration: RwLock<u32>,
}

impl Default for WeatherSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherSimulation {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(WeatherState::Clear),
            duration: RwLock::new(12000), // 10 minutes default
        }
    }

    pub fn current_state(&self) -> WeatherState {
        *self.state.read()
    }

    pub async fn set_weather(
        &self,
        new_state: WeatherState,
        duration_ticks: u32,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let old_state = *self.state.read();
        *self.state.write() = new_state;
        *self.duration.write() = duration_ticks;

        if old_state != new_state {
            self.broadcast_weather_packets(new_state, rak, players).await;
        }
    }

    pub async fn tick(
        &self,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let should_broadcast = {
            let mut dur = self.duration.write();
            if *dur > 0 {
                *dur -= 1;
                false
            } else {
                // Weather finished, return to clear
                let current = *self.state.read();
                if current != WeatherState::Clear {
                    *self.state.write() = WeatherState::Clear;
                    *dur = 24000; // 20 minutes of clear weather
                    true
                } else {
                    false
                }
            }
        };

        if should_broadcast {
            self.broadcast_weather_packets(WeatherState::Clear, rak, players).await;
        }
    }

    pub async fn send_to_player(&self, addr: SocketAddr, rak: &RakNetServer) {
        let st = *self.state.read();
        let packets = self.packets_for_state(st);
        for pk in packets {
            let batch = BatchPacket::encode(&[pk]);
            rak.send_encapsulated(addr, batch, true).await;
        }
    }

    async fn broadcast_weather_packets(
        &self,
        state: WeatherState,
        rak: &RakNetServer,
        players: &Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    ) {
        let addrs: Vec<SocketAddr> = {
            let guard = players.read();
            guard.iter().filter(|(_, s)| s.logged_in).map(|(a, _)| *a).collect()
        };
        if addrs.is_empty() {
            return;
        }

        let packets = self.packets_for_state(state);
        for pk in packets {
            let batch = BatchPacket::encode(&[pk]);
            for addr in &addrs {
                rak.send_encapsulated(*addr, batch.clone(), true).await;
            }
        }
    }

    fn packets_for_state(&self, state: WeatherState) -> Vec<Vec<u8>> {
        match state {
            WeatherState::Clear => {
                vec![
                    LevelEventPacket::new(level_event::EVENT_STOP_RAIN, 0.0, 0.0, 0.0, 0).encode(),
                    LevelEventPacket::new(level_event::EVENT_STOP_THUNDER, 0.0, 0.0, 0.0, 0).encode(),
                ]
            }
            WeatherState::Rain => {
                vec![
                    LevelEventPacket::new(level_event::EVENT_START_RAIN, 0.0, 0.0, 0.0, 10000).encode(),
                    LevelEventPacket::new(level_event::EVENT_STOP_THUNDER, 0.0, 0.0, 0.0, 0).encode(),
                ]
            }
            WeatherState::Thunder => {
                vec![
                    LevelEventPacket::new(level_event::EVENT_START_RAIN, 0.0, 0.0, 0.0, 10000).encode(),
                    LevelEventPacket::new(level_event::EVENT_START_THUNDER, 0.0, 0.0, 0.0, 10000).encode(),
                ]
            }
        }
    }
}
