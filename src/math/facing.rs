use super::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl BlockFace {
    pub const ALL: [Self; 6] = [
        Self::Down, Self::Up, Self::North, Self::South, Self::West, Self::East,
    ];

    pub fn id(self) -> u8 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => Self::Down,
            1 => Self::Up,
            2 => Self::North,
            3 => Self::South,
            4 => Self::West,
            5 => Self::East,
            _ => return None,
        })
    }

    pub fn direction(self) -> BlockPos {
        match self {
            Self::Down  => BlockPos::new(0, -1, 0),
            Self::Up    => BlockPos::new(0, 1, 0),
            Self::North => BlockPos::new(0, 0, -1),
            Self::South => BlockPos::new(0, 0, 1),
            Self::West  => BlockPos::new(-1, 0, 0),
            Self::East  => BlockPos::new(1, 0, 0),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::North | Self::South | Self::West | Self::East)
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    pub fn horizontal_angle(self) -> f32 {
        match self {
            Self::South => 0.0,
            Self::West => 90.0,
            Self::North => 180.0,
            Self::East => 270.0,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Facing;

impl Facing {
    pub const AXIS_Y: u8 = 0;
    pub const AXIS_Z: u8 = 1;
    pub const AXIS_X: u8 = 2;

    pub const FLAG_AXIS_POSITIVE: u8 = 0x08;

    pub fn flag(face: BlockFace) -> u8 {
        match face {
            BlockFace::Down  => 0x00,
            BlockFace::Up    => 0x01,
            BlockFace::North => 0x02,
            BlockFace::South => 0x03,
            BlockFace::West  => 0x04,
            BlockFace::East  => 0x05,
        }
    }

    pub fn from_flag(flag: u8) -> Option<BlockFace> {
        BlockFace::from_id(flag)
    }
}
