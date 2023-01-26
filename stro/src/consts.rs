use crate::position::Bitboard;

pub const ALL: Bitboard = 0xFFFF_FFFF_FFFF_FFFF;

pub const A_FILE: Bitboard = 0x0101_0101_0101_0101;
pub const H_FILE: Bitboard = 0x8080_8080_8080_8080;
pub const AB_FILE: Bitboard = 0x0303_0303_0303_0303;

pub const DARK_SQUARES: Bitboard = 0xAA55_AA55_AA55_AA55;
pub const LIGHT_SQUARES: Bitboard = 0x55AA_55AA_55AA_55AA;
