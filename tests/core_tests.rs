use oxide::block::BlockRegistry;
use oxide::command::CommandManager;
use oxide::item::ItemRegistry;
use oxide::inventory::CraftingManager;
use oxide::event::EventBus;
use oxide::event::event::{BlockBreakEvent, PlayerChatEvent};
use oxide::scheduler::Scheduler;
use oxide::level::chunk::Chunk;
use oxide::level::generator::{FlatGenerator, NormalGenerator, Generator};
use oxide::protocol::packet::DataPacket;
use oxide::protocol::packets::*;
use oxide::protocol::types::Item;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[test]
fn test_crafting_recipes() {
    let cm = CraftingManager::default();
    assert!(cm.count() > 0, "Crafting manager should have default recipes");

    // Test shaped recipe matching (wood to planks: 1 wood in 1x1 grid)
    let grid_wood = vec![Item::new(17, 0, 1)];
    let match_wood = cm.match_shaped(&grid_wood, 1, 1);
    assert!(match_wood.is_some(), "Wood to planks recipe should match");
    let res = match_wood.unwrap().result;
    assert_eq!(res.id, 5, "Result should be oak planks");
    assert_eq!(res.count, 4, "Result count should be 4");

    // Test furnace recipe matching (iron ore -> iron ingot)
    let iron_ore = Item::new(15, 0, 1);
    let furnace_match = cm.match_furnace(&iron_ore);
    assert!(furnace_match.is_some(), "Iron ore smelting recipe should match");
    assert_eq!(furnace_match.unwrap().result.id, 265, "Smelting iron ore should produce iron ingot");
}

#[test]
fn test_block_and_item_registry() {
    let blocks = BlockRegistry::default();
    assert!(blocks.count() > 30, "Block registry should register default vanilla blocks");

    let bedrock = blocks.get(7).expect("Bedrock should be registered");
    assert_eq!(bedrock.id(), 7);
    assert!(bedrock.hardness() < 0.0, "Bedrock should be unbreakable");

    let diamond_ore = blocks.get(56).expect("Diamond Ore should be registered");
    assert_eq!(diamond_ore.id(), 56);
    assert_eq!(diamond_ore.hardness(), 3.0);
    assert_eq!(diamond_ore.tool_type(), oxide::block::ToolType::Pickaxe);

    let items = ItemRegistry::default();
    assert!(items.count() > 30, "Item registry should register default items");

    let diamond_pickaxe = items.get(278).expect("Diamond Pickaxe should be registered");
    assert_eq!(diamond_pickaxe.id(), 278);
    assert_eq!(diamond_pickaxe.durability(), 1561);
    assert_eq!(diamond_pickaxe.tool_type(), oxide::block::ToolType::Pickaxe);
}

#[test]
fn test_event_bus_and_cancellation() {
    let bus = EventBus::new();
    bus.on("BlockBreakEvent", |ev: &mut BlockBreakEvent| {
        if ev.block_id == 7 { // Cancel breaking bedrock
            ev.cancelled = true;
        }
    });

    let ev1 = bus.fire(BlockBreakEvent {
        player_name: "Tester".to_string(),
        x: 0, y: 0, z: 0,
        block_id: 7,
        cancelled: false,
    });
    assert!(ev1.cancelled, "Bedrock breaking should be cancelled by listener");

    let ev2 = bus.fire(BlockBreakEvent {
        player_name: "Tester".to_string(),
        x: 0, y: 10, z: 0,
        block_id: 1, // Stone
        cancelled: false,
    });
    assert!(!ev2.cancelled, "Stone breaking should not be cancelled");

    // Chat event
    bus.on("PlayerChatEvent", |ev: &mut PlayerChatEvent| {
        if ev.message.contains("badword") {
            ev.cancelled = true;
        }
    });
    let chat_ev = bus.fire(PlayerChatEvent {
        player_name: "Tester".to_string(),
        message: "hello badword test".to_string(),
        cancelled: false,
    });
    assert!(chat_ev.cancelled, "Chat with forbidden words should be cancelled");
}

#[test]
fn test_scheduler() {
    let sched = Scheduler::new();
    let once_ran = Arc::new(AtomicU32::new(0));
    let rep_count = Arc::new(AtomicU32::new(0));

    let once_clone = once_ran.clone();
    sched.schedule(5, move || {
        once_clone.fetch_add(1, Ordering::SeqCst);
    });

    let rep_clone = rep_count.clone();
    let rep_id = sched.schedule_repeating(2, 3, move |_tick| {
        rep_clone.fetch_add(1, Ordering::SeqCst);
    });

    sched.tick(1);
    assert_eq!(once_ran.load(Ordering::SeqCst), 0);
    assert_eq!(rep_count.load(Ordering::SeqCst), 0);

    sched.tick(2); // repeating runs
    assert_eq!(rep_count.load(Ordering::SeqCst), 1);
    assert_eq!(once_ran.load(Ordering::SeqCst), 0);

    sched.tick(5); // repeating runs (2+3=5), once runs (at 5)
    assert_eq!(rep_count.load(Ordering::SeqCst), 2);
    assert_eq!(once_ran.load(Ordering::SeqCst), 1);

    sched.cancel(rep_id);
    sched.tick(8);
    assert_eq!(rep_count.load(Ordering::SeqCst), 2, "Cancelled task should not run again");
}

#[test]
fn test_world_generators() {
    let flat = FlatGenerator::new();
    let mut chunk_flat = Chunk::new(0, 0);
    flat.generate_chunk(&mut chunk_flat);

    assert_eq!(chunk_flat.get_block_id(0, 0, 0), 7, "y=0 should be bedrock in flat world");
    assert_eq!(chunk_flat.get_block_id(0, 1, 0), 3, "y=1 should be dirt in flat world");
    assert_eq!(chunk_flat.get_block_id(0, 2, 0), 3, "y=2 should be dirt in flat world");
    assert_eq!(chunk_flat.get_block_id(0, 3, 0), 2, "y=3 should be grass in flat world");
    assert_eq!(chunk_flat.height_map[0], 4, "Height map for flat column should be 4");
    assert_eq!(chunk_flat.biome_ids[0], 1, "Biome should be Plains (1)");

    let normal = NormalGenerator::new(12345);
    let mut chunk_normal = Chunk::new(0, 0);
    normal.generate_chunk(&mut chunk_normal);
    normal.populate(&mut chunk_normal);

    assert_eq!(chunk_normal.get_block_id(0, 0, 0), 7, "y=0 should be bedrock in normal world");
    let mut has_stone = false;
    for y in 1..60 {
        if chunk_normal.get_block_id(0, y, 0) == 1 {
            has_stone = true;
            break;
        }
    }
    assert!(has_stone, "Normal generator should produce subterranean stone layer");
}

#[test]
fn test_packet_encode_decode() {
    // ContainerOpenPacket
    let open_pk = ContainerOpenPacket::new(3, 0, 100, 64, 200);
    let encoded = open_pk.encode();
    let decoded = ContainerOpenPacket::decode(encoded);
    assert_eq!(decoded.window_id, 3);
    assert_eq!(decoded.container_type, 0);
    assert_eq!(decoded.x, 100);
    assert_eq!(decoded.y, 64);
    assert_eq!(decoded.z, 200);

    // MobEquipmentPacket
    let item = Item::new(278, 5, 1);
    let mob_eq = MobEquipmentPacket::new(42, item.clone(), 2);
    let encoded = mob_eq.encode();
    let decoded = MobEquipmentPacket::decode(encoded);
    assert_eq!(decoded.entity_runtime_id, 42);
    assert_eq!(decoded.inventory_slot, 2);
    assert_eq!(decoded.hotbar_slot, 2);
    assert_eq!(decoded.item.id, 278);

    // StartGamePacket
    let start_pk = StartGamePacket::new(1001, 1001);
    let encoded = start_pk.encode();
    let decoded = StartGamePacket::decode(encoded);
    assert_eq!(decoded.entity_unique_id, 1001);
    assert_eq!(decoded.entity_runtime_id, 1001);
    assert_eq!(decoded.difficulty, 1);

    // FullChunkDataPacket with negative coordinates (ZigZag varint verification)
    let chunk_pk = FullChunkDataPacket::new(-7, -12, vec![1, 2, 3, 4]);
    let encoded = chunk_pk.encode();
    let decoded = FullChunkDataPacket::decode(encoded);
    assert_eq!(decoded.chunk_x, -7, "Negative chunk x must decode correctly");
    assert_eq!(decoded.chunk_z, -12, "Negative chunk z must decode correctly");
    assert_eq!(decoded.data, vec![1, 2, 3, 4]);
}

#[test]
fn test_zigzag_varint_negative_values() {
    use oxide::utils::Binary;

    let test_values = [0, -1, 1, -2, 2, -100, 100, -2147483648, 2147483647];
    for &val in &test_values {
        let bytes = Binary::write_var_int(val);
        let mut offset = 0usize;
        let decoded = Binary::read_var_int(&bytes, &mut offset).expect("VarInt decoding should succeed");
        assert_eq!(decoded, val, "Round-trip VarInt mismatch for {}", val);
        assert_eq!(offset, bytes.len(), "All bytes should be consumed");
    }

    // -1 in ZigZag is encoded as 1 (single byte 0x01)
    let minus_one = Binary::write_var_int(-1);
    assert_eq!(minus_one, vec![0x01], "-1 in ZigZag must encode as 1 single byte [0x01]");

    // -2 in ZigZag is encoded as 3 (single byte 0x03)
    let minus_two = Binary::write_var_int(-2);
    assert_eq!(minus_two, vec![0x03], "-2 in ZigZag must encode as 1 single byte [0x03]");

    let test_longs = [0i64, -1i64, 1i64, -100i64, i64::MIN, i64::MAX];
    for &val in &test_longs {
        let bytes = Binary::write_var_long(val);
        let mut offset = 0usize;
        let decoded = Binary::read_var_long(&bytes, &mut offset).expect("VarLong decoding should succeed");
        assert_eq!(decoded, val, "Round-trip VarLong mismatch for {}", val);
    }
}

#[test]
fn test_creative_items_loading() {
    let items = oxide::item::creative::get_creative_items();
    assert!(!items.is_empty(), "Creative item palette must not be empty");
    assert_eq!(items.len(), 823, "MCPE 1.1.5 vanilla creative item count should be 823");
    // Verify first item is building block or tool (not all 0)
    assert!(items.iter().any(|item| item.id > 0));
}

#[test]
fn test_available_commands_packet() {
    let cm = CommandManager::new();
    let packet = cm.generate_available_commands_packet();
    assert!(!packet.commands.is_empty());
    let parsed: serde_json::Value = serde_json::from_str(&packet.commands).expect("Command palette must be valid JSON");
    assert!(parsed.get("help").is_some(), "Must contain help command");
    assert!(parsed.get("gamemode").is_some(), "Must contain gamemode command");
    assert!(parsed.get("tp").is_some() || parsed.get("teleport").is_some(), "Must contain teleport command");
}

#[test]
fn test_add_item_entity_packet_with_velocity() {
    use oxide::protocol::packets::AddItemEntityPacket;
    use oxide::protocol::packet::DataPacket;
    use oxide::protocol::types::Item;

    let mut pk = AddItemEntityPacket::new(1001, Item::new(1, 0, 1), 10.5, 64.0, -15.5);
    pk.speed_x = 0.05;
    pk.speed_y = 0.2;
    pk.speed_z = -0.08;

    let encoded = pk.encode();
    let decoded = AddItemEntityPacket::decode(encoded);

    assert_eq!(decoded.entity_runtime_id, 1001);
    assert_eq!(decoded.item.id, 1);
    assert!((decoded.x - 10.5).abs() < 1e-4);
    assert!((decoded.y - 64.0).abs() < 1e-4);
    assert!((decoded.z - (-15.5)).abs() < 1e-4);
    assert!((decoded.speed_x - 0.05).abs() < 1e-4);
    assert!((decoded.speed_y - 0.2).abs() < 1e-4);
    assert!((decoded.speed_z - (-0.08)).abs() < 1e-4);
}

#[test]
fn test_projectile_types_and_constants() {
    use oxide::entity::projectile::ProjectileKind;

    assert_eq!(ProjectileKind::EnderPearl.network_entity_type(), 87);
    assert_eq!(ProjectileKind::Snowball.network_entity_type(), 81);
    assert_eq!(ProjectileKind::Egg.network_entity_type(), 82);
    assert_eq!(ProjectileKind::Arrow.network_entity_type(), 80);
    assert_eq!(ProjectileKind::ExpBottle.network_entity_type(), 68);

    assert!((ProjectileKind::EnderPearl.gravity() - 0.03).abs() < 1e-6);
    assert!((ProjectileKind::Arrow.gravity() - 0.05).abs() < 1e-6);
    assert!((ProjectileKind::EnderPearl.drag() - 0.01).abs() < 1e-6);
}

#[tokio::test]
async fn test_liquid_simulation_spread() {
    use oxide::level::{World, LiquidSimulation};
    use oxide::raknet::RakNetServer;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::collections::HashMap;

    let world = Arc::new(RwLock::new(World::new()));
    // Set up world blocks: air at (5, 5, 5), air below at (5, 4, 5)
    {
        let mut w = world.write();
        w.set_block(5, 5, 5, 8, 0); // Water source
        w.set_block(5, 4, 5, 0, 0); // Air below
    }

    let liquid = LiquidSimulation::new();
    liquid.schedule_update(5, 5, 5);

    let rak = Arc::new(RakNetServer::new("127.0.0.1:0".parse().unwrap(), "Oxide".to_string()).await.unwrap());
    let players = Arc::new(RwLock::new(HashMap::new()));

    liquid.tick(&world, &rak, &players).await;

    // Water should flow down into (5, 4, 5)
    let below_id = {
        let mut w = world.write();
        w.get_block_id(5, 4, 5)
    };
    assert_eq!(below_id, 8, "Water should flow downward into empty air block");
}

#[tokio::test]
async fn test_liquid_horizontal_spread() {
    use oxide::level::{World, LiquidSimulation};
    use oxide::raknet::RakNetServer;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::collections::HashMap;

    let world = Arc::new(RwLock::new(World::new()));
    // Set up ground at y=3 (solid dirt = 3), water source at (10, 4, 10), air surrounding at y=4
    {
        let mut w = world.write();
        w.set_block(10, 3, 10, 3, 0); // Solid ground directly below
        w.set_block(9, 3, 10, 3, 0);
        w.set_block(11, 3, 10, 3, 0);
        w.set_block(10, 3, 9, 3, 0);
        w.set_block(10, 3, 11, 3, 0);

        w.set_block(10, 4, 10, 8, 0); // Water source
        w.set_block(9, 4, 10, 0, 0);  // Air to the west
        w.set_block(11, 4, 10, 0, 0); // Air to the east
        w.set_block(10, 4, 9, 0, 0);  // Air to the north
        w.set_block(10, 4, 11, 0, 0); // Air to the south
    }

    let liquid = LiquidSimulation::new();
    liquid.schedule_update(10, 4, 10);

    let rak = Arc::new(RakNetServer::new("127.0.0.1:0".parse().unwrap(), "Oxide".to_string()).await.unwrap());
    let players = Arc::new(RwLock::new(HashMap::new()));

    liquid.tick(&world, &rak, &players).await;

    // Verify all 4 horizontal neighbors became water (8) with decay 1
    let mut w = world.write();
    assert_eq!(w.get_block_id(9, 4, 10), 8);
    assert_eq!(w.get_block_id(11, 4, 10), 8);
    assert_eq!(w.get_block_id(10, 4, 9), 8);
    assert_eq!(w.get_block_id(10, 4, 11), 8);
}

#[tokio::test]
async fn test_liquid_lava_spread() {
    use oxide::level::{World, LiquidSimulation};
    use oxide::raknet::RakNetServer;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::collections::HashMap;

    let world = Arc::new(RwLock::new(World::new()));
    // Set up ground at y=3 (solid stone = 1), lava source at (20, 4, 20), air surrounding at y=4
    {
        let mut w = world.write();
        w.set_block(20, 3, 20, 1, 0); // Solid ground
        w.set_block(19, 3, 20, 1, 0);
        w.set_block(21, 3, 20, 1, 0);
        w.set_block(20, 3, 19, 1, 0);
        w.set_block(20, 3, 21, 1, 0);

        w.set_block(20, 4, 20, 10, 0); // Lava source
        w.set_block(19, 4, 20, 0, 0);
        w.set_block(21, 4, 20, 0, 0);
        w.set_block(20, 4, 19, 0, 0);
        w.set_block(20, 4, 21, 0, 0);
    }

    let liquid = LiquidSimulation::new();
    liquid.schedule_update(20, 4, 20);

    let rak = Arc::new(RakNetServer::new("127.0.0.1:0".parse().unwrap(), "Oxide".to_string()).await.unwrap());
    let players = Arc::new(RwLock::new(HashMap::new()));

    liquid.tick(&world, &rak, &players).await;

    // Verify all 4 horizontal neighbors became lava (10)
    let mut w = world.write();
    assert_eq!(w.get_block_id(19, 4, 20), 10);
    assert_eq!(w.get_block_id(21, 4, 20), 10);
    assert_eq!(w.get_block_id(20, 4, 19), 10);
    assert_eq!(w.get_block_id(20, 4, 21), 10);
}

#[test]
fn test_gamemode_command_parsing() {
    use oxide::command::CommandManager;

    let cmd_mgr = CommandManager::new();
    assert!(cmd_mgr.count() > 0);

    // Test that gamemode command is registered
    let cmds = cmd_mgr.all_commands();
    assert!(cmds.iter().any(|c| c.name() == "gamemode"));
}

#[test]
fn test_plugin_manager_lifecycle() {
    use oxide::plugin::{PluginManager, Plugin, PluginDescription};

    struct TestPlugin {
        desc: PluginDescription,
        enabled: bool,
    }
    impl Plugin for TestPlugin {
        fn on_enable(&mut self) { self.enabled = true; }
        fn on_disable(&mut self) { self.enabled = false; }
        fn description(&self) -> &PluginDescription { &self.desc }
    }

    let pm = PluginManager::new();
    assert_eq!(pm.count(), 0);

    let desc = PluginDescription::new("TestPL", "1.0.0");
    pm.enable(Box::new(TestPlugin { desc, enabled: false }));

    assert_eq!(pm.count(), 1);
    assert_eq!(pm.list(), vec!["TestPL".to_string()]);

    pm.disable("TestPL");
    assert_eq!(pm.count(), 0);
}

#[test]
fn test_level_sound_event_with_extra() {
    use oxide::protocol::packets::{LevelSoundEventPacket, level_event};

    let pk = LevelSoundEventPacket::new_with_extra(level_event::SOUND_BREAK, 10.0, 64.0, 20.0, 2);
    assert_eq!(pk.sound, level_event::SOUND_BREAK);
    assert_eq!(pk.extra_data, 2);
    assert_eq!(pk.x, 10.0);
    assert_eq!(pk.y, 64.0);
    assert_eq!(pk.z, 20.0);
}

#[test]
fn test_expanded_catalog() {
    let blocks = BlockRegistry::default();
    assert!(blocks.count() >= 80, "Block registry should have 80+ blocks, got {}", blocks.count());

    let items = ItemRegistry::default();
    assert!(items.count() >= 60, "Item registry should have 60+ items, got {}", items.count());
}

#[test]
fn test_hell_and_ender_generators() {
    use oxide::level::generator::{HellGenerator, EnderGenerator};

    // 1. Hell Generator test
    let hell = HellGenerator::new(12345);
    let mut hell_chunk = Chunk::new(0, 0);
    hell.generate_chunk(&mut hell_chunk);
    hell.populate(&mut hell_chunk);

    assert_eq!(hell_chunk.get_block_id(0, 0, 0), 7, "Hell bottom must be bedrock");
    assert_eq!(hell_chunk.get_block_id(0, 127, 0), 7, "Hell ceiling must be bedrock");
    assert_eq!(hell_chunk.biome_ids[0], 8, "Hell biome must be 8");

    // Check presence of netherrack (87)
    let mut has_netherrack = false;
    for y in 10..120 {
        if hell_chunk.get_block_id(8, y, 8) == 87 {
            has_netherrack = true;
            break;
        }
    }
    assert!(has_netherrack, "Hell chunk should contain netherrack");

    // 2. Ender Generator test
    let ender = EnderGenerator::new(12345);
    let mut end_chunk = Chunk::new(0, 0);
    ender.generate_chunk(&mut end_chunk);
    ender.populate(&mut end_chunk);

    assert_eq!(end_chunk.biome_ids[0], 9, "End biome must be 9");

    let mut has_endstone = false;
    for y in 45..70 {
        if end_chunk.get_block_id(8, y, 8) == 121 {
            has_endstone = true;
            break;
        }
    }
    assert!(has_endstone, "Center End chunk should contain end stone");
}

#[test]
fn test_expanded_crafting_recipes() {
    let cm = CraftingManager::default();
    assert!(cm.count() >= 45, "Crafting manager should have at least 45 recipes, got {}", cm.count());

    // Bow recipe (280 + 287)
    let bow_recipe = cm.get_recipe("craft_bow");
    assert!(bow_recipe.is_some(), "Bow recipe should exist");
    assert_eq!(bow_recipe.unwrap().result.id, 261);

    // Arrow recipe (318 + 280 + 288)
    let arrow_recipe = cm.get_recipe("craft_arrow");
    assert!(arrow_recipe.is_some(), "Arrow recipe should exist");
    let arrow = arrow_recipe.unwrap();
    assert_eq!(arrow.result.id, 262);
    assert_eq!(arrow.result.count, 4);

    // TNT recipe (289 + 12)
    let tnt_recipe = cm.get_recipe("craft_tnt");
    assert!(tnt_recipe.is_some(), "TNT recipe should exist");
    assert_eq!(tnt_recipe.unwrap().result.id, 46);

    // Golden apple recipe (266 + 260)
    let gapple_recipe = cm.get_recipe("craft_golden_apple");
    assert!(gapple_recipe.is_some(), "Golden apple recipe should exist");
    assert_eq!(gapple_recipe.unwrap().result.id, 322);

    // Bread recipe
    let bread_recipe = cm.get_recipe("craft_bread");
    assert!(bread_recipe.is_some(), "Bread recipe should exist");
    assert_eq!(bread_recipe.unwrap().result.id, 297);
}

#[test]
fn test_dimension_command_and_packets() {
    let cmd_mgr = CommandManager::new();
    let cmds = cmd_mgr.all_commands();
    assert!(cmds.iter().any(|c| c.name() == "dimension"), "Dimension command should be registered");

    // Test ChangeDimensionPacket serialization
    let pk = ChangeDimensionPacket::new(1, 100.0, 64.0, 200.0);
    assert_eq!(pk.dimension, 1);
    assert_eq!(pk.x, 100.0);
    assert_eq!(pk.y, 64.0);
    assert_eq!(pk.z, 200.0);

    let mut stream = oxide::utils::BinaryStream::new();
    pk.encode_payload(&mut stream);
    assert!(stream.len() > 0);
}

#[test]
fn test_mob_ai_math_and_yaw_pitch() {
    use oxide::entity::manager::{calculate_yaw, calculate_pitch};

    // Yaw angles: 0 is south (+z), 90 is west (-x), 180 is north (-z), 270 is east (+x)
    let yaw_north = calculate_yaw(0.0, -10.0);
    assert!((yaw_north.abs() - 180.0).abs() < 1.0 || yaw_north.abs() < 1.0);

    let yaw_east = calculate_yaw(10.0, 0.0);
    assert_eq!(yaw_east, -90.0);

    let yaw_west = calculate_yaw(-10.0, 0.0);
    assert_eq!(yaw_west, 90.0);

    // Pitch angles: looking straight ahead is 0, up is negative, down is positive
    let pitch_level = calculate_pitch(0.0, 64.0, 0.0, 10.0, 64.0, 0.0);
    assert_eq!(pitch_level, 0.0);

    let pitch_up = calculate_pitch(0.0, 64.0, 0.0, 0.0, 74.0, 0.0);
    assert!(pitch_up < -80.0);

    let pitch_down = calculate_pitch(0.0, 64.0, 0.0, 0.0, 54.0, 0.0);
    assert!(pitch_down > 80.0);
}

#[test]
fn test_mob_loot_tables_vanilla_parity() {
    use oxide::entity::LootTable;

    // Test Zombie drops (Rotten flesh: 367)
    let zombie_drops = LootTable::get_drops(32, true, false, 0);
    for item in &zombie_drops {
        assert!(item.id == 367 || item.id == 265 || item.id == 391 || item.id == 392);
    }

    // Test Skeleton drops (Arrow: 262, Bone: 352)
    let skeleton_drops = LootTable::get_drops(34, true, false, 0);
    for item in &skeleton_drops {
        assert!(item.id == 262 || item.id == 352);
    }

    // Test Wither drops guaranteed Nether Star (399)
    let wither_drops = LootTable::get_drops(52, true, false, 0);
    assert_eq!(wither_drops.len(), 1);
    assert_eq!(wither_drops[0].id, 399);

    // Test Evoker drops guaranteed Totem of Undying (449)
    let evoker_drops = LootTable::get_drops(58, true, false, 0);
    assert!(evoker_drops.iter().any(|i| i.id == 449));
}

#[test]
fn test_effect_container_and_mechanics() {
    use oxide::entity::{EffectContainer, ActiveEffect, effect_id};

    let mut container = EffectContainer::new();
    container.add_effect(ActiveEffect::new(effect_id::SPEED, 200, 1));
    assert!(container.has_effect(effect_id::SPEED));
    assert_eq!(container.speed_multiplier(), 1.4); // Level 2 speed (+40%)

    // Add absorption
    container.add_effect(ActiveEffect::new(effect_id::ABSORPTION, 1000, 0));
    assert_eq!(container.absorption_hp, 4.0);

    // Add fire resistance
    container.add_effect(ActiveEffect::new(effect_id::FIRE_RESISTANCE, 500, 0));
    assert!(container.has_fire_resistance());

    // Tick effects
    let (expired, _) = container.tick();
    assert!(expired.is_empty());
    assert_eq!(container.get_effect(effect_id::SPEED).unwrap().duration, 199);
}

