use std::collections::HashMap;

/// Effect IDs matching vanilla Minecraft Bedrock 1.1.5
pub mod effect_id {
    pub const SPEED: u8          = 1;
    pub const SLOWNESS: u8       = 2;
    pub const HASTE: u8          = 3;
    pub const MINING_FATIGUE: u8 = 4;
    pub const STRENGTH: u8       = 5;
    pub const INSTANT_HEALTH: u8 = 6;
    pub const INSTANT_DAMAGE: u8 = 7;
    pub const JUMP_BOOST: u8     = 8;
    pub const NAUSEA: u8         = 9;
    pub const REGENERATION: u8   = 10;
    pub const RESISTANCE: u8     = 11;
    pub const FIRE_RESISTANCE: u8= 12;
    pub const WATER_BREATHING: u8= 13;
    pub const INVISIBILITY: u8   = 14;
    pub const BLINDNESS: u8      = 15;
    pub const NIGHT_VISION: u8   = 16;
    pub const HUNGER: u8         = 17;
    pub const WEAKNESS: u8       = 18;
    pub const POISON: u8         = 19;
    pub const WITHER: u8         = 20;
    pub const HEALTH_BOOST: u8   = 21;
    pub const ABSORPTION: u8     = 22;
    pub const SATURATION: u8     = 23;
}

/// A single active effect instance on an entity or player.
#[derive(Debug, Clone)]
pub struct ActiveEffect {
    pub id: u8,
    /// Remaining duration in ticks (20 ticks = 1 second).
    pub duration: i32,
    /// Amplifier (0 = level I, 1 = level II, etc.).
    pub amplifier: u8,
    /// Whether the effect shows particles.
    pub show_particles: bool,
}

impl ActiveEffect {
    pub fn new(id: u8, duration_ticks: i32, amplifier: u8) -> Self {
        Self {
            id,
            duration: duration_ticks,
            amplifier,
            show_particles: true,
        }
    }

    /// Returns true if this is an instant effect (applied once, not over time).
    pub fn is_instant(&self) -> bool {
        matches!(self.id, effect_id::INSTANT_HEALTH | effect_id::INSTANT_DAMAGE)
    }
}

/// Per-player effect container — stored in PlayerState.
#[derive(Debug, Clone, Default)]
pub struct EffectContainer {
    effects: HashMap<u8, ActiveEffect>,
    /// Extra absorption hearts (from Absorption effect).
    pub absorption_hp: f32,
}

impl EffectContainer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or refresh an effect. If the same effect already exists with a lower
    /// or equal duration/amplifier, it is replaced.
    pub fn add_effect(&mut self, effect: ActiveEffect) {
        if let Some(existing) = self.effects.get(&effect.id) {
            // Only override if new effect is stronger or longer
            if effect.amplifier < existing.amplifier {
                return;
            }
            if effect.amplifier == existing.amplifier && effect.duration <= existing.duration {
                return;
            }
        }
        // Handle Absorption — set extra HP
        if effect.id == effect_id::ABSORPTION {
            let hearts = 4.0 * (effect.amplifier as f32 + 1.0);
            if hearts > self.absorption_hp {
                self.absorption_hp = hearts;
            }
        }
        self.effects.insert(effect.id, effect);
    }

    pub fn remove_effect(&mut self, id: u8) {
        self.effects.remove(&id);
        if id == effect_id::ABSORPTION {
            self.absorption_hp = 0.0;
        }
    }

    pub fn has_effect(&self, id: u8) -> bool {
        self.effects.contains_key(&id)
    }

    pub fn get_effect(&self, id: u8) -> Option<&ActiveEffect> {
        self.effects.get(&id)
    }

    pub fn all_effects(&self) -> Vec<ActiveEffect> {
        self.effects.values().cloned().collect()
    }

    /// Tick effects down by 1. Returns a list of effects that expired this tick,
    /// and healing/damage values to apply to the entity.
    ///
    /// Returns: (expired_ids, hp_delta)
    pub fn tick(&mut self) -> (Vec<u8>, f32) {
        let mut expired = Vec::new();
        let mut hp_delta = 0.0f32;

        for (id, eff) in self.effects.iter_mut() {
            eff.duration -= 1;

            match eff.id {
                effect_id::REGENERATION => {
                    // Regen I heals 1 HP every 50 ticks, Regen II every 25, etc.
                    let interval = (50u32) >> (eff.amplifier as u32).min(4);
                    // duration starts counting down — heal when (duration % interval == 0)
                    if eff.duration >= 0 && (eff.duration as u32) % interval.max(1) == 0 {
                        hp_delta += 1.0;
                    }
                }
                effect_id::POISON => {
                    // Poison ticks every 25 ticks (level I), 12 (level II)
                    let interval = (25u32) >> (eff.amplifier as u32).min(4);
                    if eff.duration >= 0 && (eff.duration as u32) % interval.max(1) == 0 {
                        hp_delta -= 1.0;
                    }
                }
                effect_id::WITHER => {
                    // Wither ticks every 40 ticks
                    let interval = (40u32) >> (eff.amplifier as u32).min(4);
                    if eff.duration >= 0 && (eff.duration as u32) % interval.max(1) == 0 {
                        hp_delta -= 1.0;
                    }
                }
                _ => {}
            }

            if eff.duration <= 0 {
                expired.push(*id);
            }
        }

        for id in &expired {
            self.effects.remove(id);
            if *id == effect_id::ABSORPTION {
                self.absorption_hp = 0.0;
            }
        }

        (expired, hp_delta)
    }

    /// Returns the speed multiplier from Speed/Slowness effects.
    pub fn speed_multiplier(&self) -> f32 {
        let speed_amp = self.effects.get(&effect_id::SPEED)
            .map(|e| e.amplifier as f32 + 1.0)
            .unwrap_or(0.0);
        let slow_amp = self.effects.get(&effect_id::SLOWNESS)
            .map(|e| e.amplifier as f32 + 1.0)
            .unwrap_or(0.0);
        1.0 + (speed_amp * 0.2) - (slow_amp * 0.15)
    }

    /// Returns the damage multiplier from Strength/Weakness.
    pub fn damage_multiplier(&self) -> f32 {
        let str_amp = self.effects.get(&effect_id::STRENGTH)
            .map(|e| e.amplifier as f32 + 1.0)
            .unwrap_or(0.0);
        let weak_amp = self.effects.get(&effect_id::WEAKNESS)
            .map(|e| e.amplifier as f32 + 1.0)
            .unwrap_or(0.0);
        1.0 + (str_amp * 0.3) - (weak_amp * 0.2)
    }

    /// Returns true if the player has Fire Resistance active.
    pub fn has_fire_resistance(&self) -> bool {
        self.has_effect(effect_id::FIRE_RESISTANCE)
    }

    /// Returns true if the player has Water Breathing active.
    pub fn has_water_breathing(&self) -> bool {
        self.has_effect(effect_id::WATER_BREATHING)
    }

    /// Returns resistance damage reduction factor (0.0 = none, 1.0 = immune).
    pub fn resistance_reduction(&self) -> f32 {
        self.effects.get(&effect_id::RESISTANCE)
            .map(|e| (e.amplifier as f32 + 1.0) * 0.2)
            .unwrap_or(0.0)
            .min(1.0)
    }

    /// Apply instant effects and return the hp delta.
    pub fn apply_instant_health(&mut self, amplifier: u8) -> f32 {
        let heal = (2.0f32).powi(amplifier as i32 + 1) * 2.0;
        heal
    }

    pub fn apply_instant_damage(&mut self, amplifier: u8) -> f32 {
        let dmg = (2.0f32).powi(amplifier as i32 + 1) * 3.0;
        -dmg
    }
}

/// Helper: build golden apple effects (Regen II 5s, Absorption I 2min).
pub fn golden_apple_effects() -> Vec<ActiveEffect> {
    vec![
        ActiveEffect::new(effect_id::REGENERATION, 100, 1), // Regen II, 5 seconds
        ActiveEffect::new(effect_id::ABSORPTION, 2400, 0),  // Absorption I, 2 minutes
    ]
}

/// Helper: build milk bucket effect (clears all effects).
/// Returns empty vec — caller must clear all effects on player.
pub fn milk_clears_effects() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_add_and_tick() {
        let mut container = EffectContainer::new();
        container.add_effect(ActiveEffect::new(effect_id::REGENERATION, 50, 0));
        assert!(container.has_effect(effect_id::REGENERATION));

        let (_expired, _hp) = container.tick();
        // After 1 tick, still active
        assert!(container.has_effect(effect_id::REGENERATION));
    }

    #[test]
    fn test_effect_expiry() {
        let mut container = EffectContainer::new();
        container.add_effect(ActiveEffect::new(effect_id::SPEED, 1, 0));
        let (expired, _) = container.tick();
        assert!(expired.contains(&effect_id::SPEED));
        assert!(!container.has_effect(effect_id::SPEED));
    }

    #[test]
    fn test_golden_apple_effects() {
        let effects = golden_apple_effects();
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].id, effect_id::REGENERATION);
        assert_eq!(effects[0].amplifier, 1); // Regen II
        assert_eq!(effects[1].id, effect_id::ABSORPTION);
    }

    #[test]
    fn test_absorption_hp() {
        let mut container = EffectContainer::new();
        container.add_effect(ActiveEffect::new(effect_id::ABSORPTION, 2400, 0));
        assert_eq!(container.absorption_hp, 4.0); // 4 extra HP (2 hearts)
    }
}
