use rand::Rng;
use crate::entity::effect::{effect_id, ActiveEffect};
use crate::item::enchantment::{
    parse_enchantments_from_nbt, should_consume_durability,
    TYPE_ARMOR_EXPLOSION_PROTECTION, TYPE_ARMOR_FALL_PROTECTION, TYPE_ARMOR_FIRE_PROTECTION,
    TYPE_ARMOR_PROJECTILE_PROTECTION, TYPE_ARMOR_PROTECTION, TYPE_ARMOR_THORNS,
    TYPE_MINING_DURABILITY,
};
use crate::protocol::handler::PlayerState;
use crate::protocol::types::Item;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageCause {
    Contact,
    EntityAttack,
    Projectile,
    Suffocation,
    Fall,
    Fire,
    FireTick,
    Lava,
    Drowning,
    BlockExplosion,
    EntityExplosion,
    Void,
    Suicide,
    Magic,
    Starvation,
}

/// Returns `(defense_points, max_durability)` for armor item IDs.
pub fn armor_stats(item_id: i32) -> (u32, i32) {
    match item_id {
        298 => (1, 56),
        299 => (3, 81),
        300 => (2, 76),
        301 => (1, 66),
        302 => (1, 166),
        303 => (5, 241),
        304 => (4, 226),
        305 => (1, 196),
        306 => (2, 166),
        307 => (6, 241),
        308 => (5, 226),
        309 => (2, 196),
        310 => (3, 364),
        311 => (8, 529),
        312 => (6, 496),
        313 => (3, 430),
        314 => (1, 78),
        315 => (5, 113),
        316 => (3, 106),
        317 => (1, 92),
        444 => (0, 431),
        _ => (0, 0),
    }
}

#[derive(Debug, Clone)]
pub struct DamageOutcome {
    pub final_damage: f32,
    pub absorbed: f32,
    pub thorns_damage: f32,
    pub totem_used: bool,
    pub died: bool,
    pub armor_changed: bool,
    pub armor_broke: bool,
}

/// Full player damage reduction pipeline: Resistance, Armor Points, Enchantment EPF,
/// Thorns reflection, Absorption buffer, Armor durability wear, and Totem of Undying.
pub fn apply_player_damage(
    state: &mut PlayerState,
    raw_damage: f32,
    cause: DamageCause,
    is_critical: bool,
) -> DamageOutcome {
    let mut result = DamageOutcome {
        final_damage: 0.0,
        absorbed: 0.0,
        thorns_damage: 0.0,
        totem_used: false,
        died: false,
        armor_changed: false,
        armor_broke: false,
    };

    if (state.gamemode == 1 || state.gamemode == 3)
        && cause != DamageCause::Void
        && cause != DamageCause::Suicide
    {
        return result;
    }

    if state.effects.has_fire_resistance()
        && matches!(cause, DamageCause::Fire | DamageCause::FireTick | DamageCause::Lava)
    {
        return result;
    }

    let mut dmg = raw_damage.max(0.0);
    if is_critical {
        dmg *= 1.5;
    }

    // Resistance effect
    if cause != DamageCause::Void && cause != DamageCause::Suicide {
        if let Some(res) = state.effects.get_effect(effect_id::RESISTANCE) {
            let mult = (1.0 - 0.20 * (res.amplifier as f32 + 1.0)).max(0.0);
            dmg *= mult;
        }
    }

    let armor_applies = matches!(
        cause,
        DamageCause::Contact
            | DamageCause::EntityAttack
            | DamageCause::Projectile
            | DamageCause::Fire
            | DamageCause::Lava
            | DamageCause::BlockExplosion
            | DamageCause::EntityExplosion
    );

    let mut armor = [
        state.inventory.helmet(),
        state.inventory.chestplate(),
        state.inventory.leggings(),
        state.inventory.boots(),
    ];

    let mut total_points = 0u32;
    let mut raw_epf = 0u32;
    let mut max_thorns_lvl = 0u16;

    for piece in &armor {
        if piece.is_air() || piece.count == 0 {
            continue;
        }
        let (pts, _) = armor_stats(piece.id);
        total_points += pts;

        let enchs = parse_enchantments_from_nbt(&piece.nbt);
        for e in &enchs {
            match e.id {
                TYPE_ARMOR_PROTECTION => raw_epf += e.level as u32,
                TYPE_ARMOR_FIRE_PROTECTION
                    if matches!(cause, DamageCause::Fire | DamageCause::FireTick | DamageCause::Lava) =>
                {
                    raw_epf += (e.level as u32) * 2;
                }
                TYPE_ARMOR_FALL_PROTECTION if cause == DamageCause::Fall => {
                    raw_epf += (e.level as u32) * 3;
                }
                TYPE_ARMOR_EXPLOSION_PROTECTION
                    if matches!(cause, DamageCause::BlockExplosion | DamageCause::EntityExplosion) =>
                {
                    raw_epf += (e.level as u32) * 2;
                }
                TYPE_ARMOR_PROJECTILE_PROTECTION if cause == DamageCause::Projectile => {
                    raw_epf += (e.level as u32) * 2;
                }
                TYPE_ARMOR_THORNS if cause == DamageCause::EntityAttack => {
                    max_thorns_lvl = max_thorns_lvl.max(e.level);
                }
                _ => {}
            }
        }
    }

    if armor_applies && total_points > 0 {
        let armor_mult = (1.0 - 0.04 * (total_points.min(20) as f32)).max(0.2);
        dmg *= armor_mult;
    }

    let epf_capped_mult = {
        let mut rng = rand::thread_rng();
        if raw_epf > 0 && cause != DamageCause::Void && cause != DamageCause::Suicide && cause != DamageCause::Starvation {
            let scaled = ((raw_epf.min(25) as f32) * (rng.gen_range(50..=100) as f32 / 100.0)).ceil() as u32;
            let epf = scaled.min(20);
            1.0 - 0.04 * epf as f32
        } else {
            1.0
        }
    };
    dmg *= epf_capped_mult;

    // Thorns reflection
    if max_thorns_lvl > 0 && cause == DamageCause::EntityAttack {
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < (0.15 * max_thorns_lvl as f32) {
            result.thorns_damage = rng.gen_range(1..=4) as f32;
        }
    }

    // Damage armor pieces
    if armor_applies && raw_damage > 0.0 {
        let dur_hit = ((raw_damage / 4.0).floor() as i32).max(1);
        let mut rng = rand::thread_rng();
        for (idx, piece) in armor.iter_mut().enumerate() {
            let (_, max_dur) = armor_stats(piece.id);
            if piece.is_air() || max_dur <= 0 {
                continue;
            }
            let unbreaking = parse_enchantments_from_nbt(&piece.nbt)
                .iter()
                .find(|e| e.id == TYPE_MINING_DURABILITY)
                .map(|e| e.level)
                .unwrap_or(0);
            let mut extra = 0;
            if result.thorns_damage > 0.0
                && parse_enchantments_from_nbt(&piece.nbt).iter().any(|e| e.id == TYPE_ARMOR_THORNS)
            {
                extra = 2;
            }
            let total_wear = dur_hit + extra;
            let mut applied = 0;
            for _ in 0..total_wear {
                if should_consume_durability(unbreaking, true, rng.gen()) {
                    applied += 1;
                }
            }
            if applied > 0 {
                piece.meta += applied;
                result.armor_changed = true;
                if piece.meta >= max_dur {
                    *piece = Item::air();
                    result.armor_broke = true;
                }
                state.inventory.set_armor(idx, piece.clone());
            }
        }
    }

    result.final_damage = dmg;

    // Consume Absorption shield first
    if state.effects.absorption_hp > 0.0 && dmg > 0.0 {
        let used = state.effects.absorption_hp.min(dmg);
        state.effects.absorption_hp -= used;
        dmg -= used;
        result.absorbed = used;
    }

    let hp_loss = dmg.floor() as u32;
    let remainder = dmg - hp_loss as f32;
    let total_sub = if remainder >= 0.5 { hp_loss + 1 } else { hp_loss };
    state.health = state.health.saturating_sub(total_sub);

    // Check Totem of Undying resurrection
    if state.health == 0 && cause != DamageCause::Void {
        let offhand = state.inventory.offhand();
        let held = state.inventory.held_item();
        let held_slot = state.inventory.held_slot();
        if offhand.id == 450 {
            state.inventory.set_offhand(Item::air());
            revive_with_totem(state);
            result.totem_used = true;
        } else if held.id == 450 {
            let mut replacement = held.clone();
            if replacement.count > 1 {
                replacement.count -= 1;
            } else {
                replacement = Item::air();
            }
            state.inventory.set_item(held_slot, replacement);
            revive_with_totem(state);
            result.totem_used = true;
        } else {
            result.died = true;
            for it in state.inventory.get_contents() {
                if !it.is_air() && it.count > 0 {
                    state.pending_drops.push(it);
                }
            }
            for arm in state.inventory.armor_contents() {
                if !arm.is_air() && arm.count > 0 {
                    state.pending_drops.push(arm);
                }
            }
            let off = state.inventory.offhand();
            if !off.is_air() && off.count > 0 {
                state.pending_drops.push(off);
            }
            state.inventory.set_contents(vec![Item::air(); 36]);
            state.inventory.set_armor_contents(vec![Item::air(); 4]);
            state.inventory.set_offhand(Item::air());
            state.pending_xp_drop += (state.xp_level * 7).min(100);
            state.xp_level = 0;
            state.xp_points = 0;
            state.xp_progress = 0.0;
        }
    }

    result
}

fn revive_with_totem(state: &mut PlayerState) {
    state.health = 1;
    state.effects = crate::entity::effect::EffectContainer::new();
    state.effects.add_effect(ActiveEffect::new(effect_id::REGENERATION, 900, 1));
    state.effects.add_effect(ActiveEffect::new(effect_id::FIRE_RESISTANCE, 800, 0));
    state.effects.add_effect(ActiveEffect::new(effect_id::ABSORPTION, 100, 1));
}

/// Applies player damage inside lock, then broadcasts health, armor sync, knockback,
/// Totem resurrection visuals/effects, and death animation without holding locks across `.await`.
pub async fn hurt_player(
    players: &std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<std::net::SocketAddr, PlayerState>>>,
    rak: &crate::raknet::RakNetServer,
    target_addr: std::net::SocketAddr,
    raw_damage: f32,
    cause: DamageCause,
    is_critical: bool,
    attacker_pos: Option<(f64, f64, f64)>,
    sound_override: Option<u8>,
    death_msg: Option<&str>,
) -> DamageOutcome {
    use crate::protocol::packet::DataPacket;
    use crate::protocol::batch::BatchPacket;
    use crate::protocol::packets::{
        AnimatePacket, Attribute, ContainerSetContentPacket, ContainerSetSlotPacket,
        EntityEventPacket, LevelEventPacket, LevelSoundEventPacket, MobArmorEquipmentPacket,
        MobEffectPacket, SetEntityMotionPacket, SetHealthPacket, TextPacket,
        UpdateAttributesPacket, level_event,
    };
    use crate::protocol::types::ContainerIds;

    struct PostSync {
        eid: i64,
        dim: i32,
        px: f64,
        py: f64,
        pz: f64,
        new_hp: u32,
        outcome: DamageOutcome,
        armor: [Item; 4],
        offhand: Item,
        held_slot: usize,
        held_item: Item,
        peers: Vec<std::net::SocketAddr>,
        all_addrs: Vec<std::net::SocketAddr>,
    }

    let sync_opt: Option<PostSync> = {
        let mut guard = players.write();
        let outcome;
        let eid;
        let dim;
        let px;
        let py;
        let pz;
        let new_hp;
        let armor;
        let offhand;
        let held_slot;
        let held_item;
        {
            let s = match guard.get_mut(&target_addr) {
                Some(s) if s.spawned && s.health > 0 => s,
                _ => return DamageOutcome {
                    final_damage: 0.0,
                    absorbed: 0.0,
                    thorns_damage: 0.0,
                    totem_used: false,
                    died: false,
                    armor_changed: false,
                    armor_broke: false,
                },
            };

            if s.no_damage_ticks > 0 && !matches!(cause, DamageCause::Void | DamageCause::Suicide) {
                return DamageOutcome {
                    final_damage: 0.0,
                    absorbed: 0.0,
                    thorns_damage: 0.0,
                    totem_used: false,
                    died: false,
                    armor_changed: false,
                    armor_broke: false,
                };
            }

            outcome = apply_player_damage(s, raw_damage, cause, is_critical);
            if outcome.final_damage > 0.0 || outcome.absorbed > 0.0 || outcome.totem_used {
                s.no_damage_ticks = 10;
            }
            eid = s.entity_id;
            dim = s.dimension;
            px = s.x;
            py = s.y;
            pz = s.z;
            new_hp = s.health;
            armor = [
                s.inventory.helmet(),
                s.inventory.chestplate(),
                s.inventory.leggings(),
                s.inventory.boots(),
            ];
            offhand = s.inventory.offhand();
            held_slot = s.inventory.held_slot();
            held_item = s.inventory.held_item();
        }
        let peers = guard
            .iter()
            .filter(|(a, ps)| **a != target_addr && ps.logged_in && ps.dimension == dim)
            .map(|(a, _)| *a)
            .collect();
        let all_addrs = guard
            .iter()
            .filter(|(_, ps)| ps.logged_in)
            .map(|(a, _)| *a)
            .collect();

        Some(PostSync {
            eid,
            dim,
            px,
            py,
            pz,
            new_hp,
            outcome,
            armor,
            offhand,
            held_slot,
            held_item,
            peers,
            all_addrs,
        })
    };

    let sync = match sync_opt {
        Some(s) => s,
        None => unreachable!(),
    };
    let _ = sync.dim;

    if sync.outcome.final_damage <= 0.0 && sync.outcome.absorbed <= 0.0 && !sync.outcome.totem_used {
        return sync.outcome;
    }

    let mut self_pks = Vec::new();
    self_pks.push(SetHealthPacket::new(sync.new_hp as i32).encode());
    let mut attrs = UpdateAttributesPacket::new(sync.eid);
    attrs.attributes = vec![Attribute::health(sync.new_hp as f32)];
    self_pks.push(attrs.encode());

    if sync.outcome.armor_changed {
        let mut arm_content = ContainerSetContentPacket::new(ContainerIds::ARMOR as u32);
        arm_content.items = sync.armor.to_vec();
        self_pks.push(arm_content.encode());

        let mut mob_arm = MobArmorEquipmentPacket::new(sync.eid);
        mob_arm.helmet = sync.armor[0].clone();
        mob_arm.chestplate = sync.armor[1].clone();
        mob_arm.leggings = sync.armor[2].clone();
        mob_arm.boots = sync.armor[3].clone();
        let encoded_arm = BatchPacket::encode(&[mob_arm.encode()]);
        for &p in &sync.peers {
            rak.send_encapsulated(p, encoded_arm.clone(), true).await;
        }
        if sync.outcome.armor_broke {
            let brk = LevelSoundEventPacket::new(
                level_event::SOUND_BREAK,
                sync.px as f32,
                sync.py as f32,
                sync.pz as f32,
            );
            let brk_batch = BatchPacket::encode(&[brk.encode()]);
            rak.send_encapsulated(target_addr, brk_batch.clone(), true).await;
            for &p in &sync.peers {
                rak.send_encapsulated(p, brk_batch.clone(), true).await;
            }
        }
    }

    if sync.outcome.totem_used {
        let mut off_pk = ContainerSetContentPacket::new(ContainerIds::OFFHAND as u32);
        off_pk.items = vec![sync.offhand];
        self_pks.push(off_pk.encode());
        self_pks.push(ContainerSetSlotPacket::new(0, sync.held_slot as i32, sync.held_item).encode());

        for (eff_id, amp, dur) in [
            (effect_id::REGENERATION, 1, 900),
            (effect_id::FIRE_RESISTANCE, 0, 800),
            (effect_id::ABSORPTION, 1, 100),
        ] {
            let eff_pk = MobEffectPacket {
                entity_runtime_id: sync.eid,
                event_id: crate::protocol::packets::entity_packets::EVENT_ADD,
                effect_id: eff_id as i32,
                amplifier: amp,
                particles: true,
                duration: dur,
            };
            self_pks.push(eff_pk.encode());
        }

        let ev_pk = EntityEventPacket::new(sync.eid, EntityEventPacket::CONSUME_TOTEM);
        let snd_pk = LevelEventPacket::new(
            level_event::EVENT_SOUND_TOTEM,
            sync.px as f32,
            sync.py as f32,
            sync.pz as f32,
            0,
        );
        self_pks.push(ev_pk.encode());
        self_pks.push(snd_pk.encode());
        let peer_totem = BatchPacket::encode(&[ev_pk.encode(), snd_pk.encode()]);
        for &p in &sync.peers {
            rak.send_encapsulated(p, peer_totem.clone(), true).await;
        }
    } else {
        let anim = AnimatePacket::new(1, sync.eid);
        let snd_id = sound_override.unwrap_or(level_event::SOUND_HURT);
        let hurt_snd = LevelSoundEventPacket::new(
            snd_id,
            sync.px as f32,
            sync.py as f32,
            sync.pz as f32,
        );
        self_pks.push(anim.encode());
        self_pks.push(hurt_snd.encode());
        let peer_hurt = BatchPacket::encode(&[anim.encode(), hurt_snd.encode()]);
        for &p in &sync.peers {
            rak.send_encapsulated(p, peer_hurt.clone(), true).await;
        }

        if let Some((ax, _ay, az)) = attacker_pos {
            if !sync.outcome.died {
                let dx = sync.px - ax;
                let dz = sync.pz - az;
                let len = (dx * dx + dz * dz).sqrt().max(0.01);
                let kx = ((dx / len) * 0.4) as f32;
                let kz = ((dz / len) * 0.4) as f32;
                let mot = SetEntityMotionPacket::new(sync.eid, kx, 0.36, kz);
                self_pks.push(mot.encode());
                let peer_mot = BatchPacket::encode(&[mot.encode()]);
                for &p in &sync.peers {
                    rak.send_encapsulated(p, peer_mot.clone(), true).await;
                }
            }
        }
    }

    if sync.outcome.died {
        let mut clear_inv = ContainerSetContentPacket::new(ContainerIds::INVENTORY as u32);
        clear_inv.target_eid = sync.eid;
        clear_inv.items = vec![Item::air(); 36];
        let mut clear_arm = ContainerSetContentPacket::new(ContainerIds::ARMOR as u32);
        clear_arm.target_eid = sync.eid;
        clear_arm.items = vec![Item::air(); 4];
        let mut clear_off = ContainerSetContentPacket::new(ContainerIds::OFFHAND as u32);
        clear_off.target_eid = sync.eid;
        clear_off.items = vec![Item::air()];
        let mut xp_attrs = UpdateAttributesPacket::new(sync.eid);
        xp_attrs.attributes = vec![
            Attribute::experience_level(0.0),
            Attribute::experience_progress(0.0),
        ];
        self_pks.push(clear_inv.encode());
        self_pks.push(clear_arm.encode());
        self_pks.push(clear_off.encode());
        self_pks.push(xp_attrs.encode());

        let mob_arm = MobArmorEquipmentPacket::new(sync.eid);
        let death_anim = EntityEventPacket::new(sync.eid, EntityEventPacket::DEATH_ANIMATION);
        let peer_death = BatchPacket::encode(&[death_anim.encode(), mob_arm.encode()]);
        for &p in &sync.peers {
            rak.send_encapsulated(p, peer_death.clone(), true).await;
        }
        if let Some(msg) = death_msg {
            let txt = BatchPacket::encode(&[TextPacket::system(msg.to_string()).encode()]);
            for &a in &sync.all_addrs {
                rak.send_encapsulated(a, txt.clone(), true).await;
            }
        }
    }

    rak.send_encapsulated(target_addr, BatchPacket::encode(&self_pks), true).await;
    sync.outcome
}
