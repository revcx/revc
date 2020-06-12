use super::plane::*;

pub const SUBPEL_FILTER_SIZE: usize = 8;

pub const PLANES: usize = 3;

pub const MI_SIZE_LOG2: usize = 2;
pub const MI_SIZE: usize = (1 << MI_SIZE_LOG2);
const MAX_MIB_SIZE_LOG2: usize = (MAX_SB_SIZE_LOG2 - MI_SIZE_LOG2);
pub const MAX_MIB_SIZE: usize = (1 << MAX_MIB_SIZE_LOG2);
pub const MAX_MIB_MASK: usize = (MAX_MIB_SIZE - 1);

const MAX_SB_SIZE_LOG2: usize = 6;
pub const MAX_SB_SIZE: usize = (1 << MAX_SB_SIZE_LOG2);
const MAX_SB_SQUARE: usize = (MAX_SB_SIZE * MAX_SB_SIZE);

const SUPERBLOCK_TO_PLANE_SHIFT: usize = MAX_SB_SIZE_LOG2;
const SUPERBLOCK_TO_BLOCK_SHIFT: usize = MAX_MIB_SIZE_LOG2;
pub const BLOCK_TO_PLANE_SHIFT: usize = MI_SIZE_LOG2;
pub const LOCAL_BLOCK_MASK: usize = (1 << SUPERBLOCK_TO_BLOCK_SHIFT) - 1;

/// Absolute offset in superblocks inside a plane, where a superblock is defined
/// to be an N*N square where N = (1 << SUPERBLOCK_TO_PLANE_SHIFT).
#[derive(Clone, Copy, Debug)]
pub struct SuperBlockOffset {
    pub x: usize,
    pub y: usize,
}

impl SuperBlockOffset {
    /// Offset of a block inside the current superblock.
    pub fn block_offset(self, block_x: usize, block_y: usize) -> BlockOffset {
        BlockOffset {
            x: (self.x << SUPERBLOCK_TO_BLOCK_SHIFT) + block_x,
            y: (self.y << SUPERBLOCK_TO_BLOCK_SHIFT) + block_y,
        }
    }

    /// Offset of the top-left pixel of this block.
    pub fn plane_offset(self, plane: &PlaneConfig) -> PlaneOffset {
        PlaneOffset {
            x: (self.x as isize) << (SUPERBLOCK_TO_PLANE_SHIFT - plane.xdec),
            y: (self.y as isize) << (SUPERBLOCK_TO_PLANE_SHIFT - plane.ydec),
        }
    }
}

/// Absolute offset in blocks inside a plane, where a block is defined
/// to be an N*N square where N = (1 << BLOCK_TO_PLANE_SHIFT).
#[derive(Clone, Copy, Debug)]
pub struct BlockOffset {
    pub x: usize,
    pub y: usize,
}

impl BlockOffset {
    /// Offset of the superblock in which this block is located.
    pub fn sb_offset(self) -> SuperBlockOffset {
        SuperBlockOffset {
            x: self.x >> SUPERBLOCK_TO_BLOCK_SHIFT,
            y: self.y >> SUPERBLOCK_TO_BLOCK_SHIFT,
        }
    }

    /// Offset of the top-left pixel of this block.
    pub fn plane_offset(self, plane: &PlaneConfig) -> PlaneOffset {
        PlaneOffset {
            x: (self.x >> plane.xdec << BLOCK_TO_PLANE_SHIFT) as isize,
            y: (self.y >> plane.ydec << BLOCK_TO_PLANE_SHIFT) as isize,
        }
    }

    /// Convert to plane offset without decimation
    #[inline]
    pub fn to_luma_plane_offset(self) -> PlaneOffset {
        PlaneOffset {
            x: (self.x as isize) << BLOCK_TO_PLANE_SHIFT,
            y: (self.y as isize) << BLOCK_TO_PLANE_SHIFT,
        }
    }

    pub fn y_in_sb(self) -> usize {
        self.y % MAX_MIB_SIZE
    }

    pub fn with_offset(self, col_offset: isize, row_offset: isize) -> BlockOffset {
        let x = self.x as isize + col_offset;
        let y = self.y as isize + row_offset;
        debug_assert!(x >= 0);
        debug_assert!(y >= 0);

        BlockOffset {
            x: x as usize,
            y: y as usize,
        }
    }
}
