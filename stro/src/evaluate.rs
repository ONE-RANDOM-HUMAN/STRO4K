// use crate::consts;
use crate::position::{Bitboard, Board, Color};
use std::arch::x86_64::*;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct Eval(i16, i16);

pub const MAX_EVAL: i32 = 128 * 256 - 1;
pub const MIN_EVAL: i32 = -MAX_EVAL;

const NN: [f32; 6333] = unsafe {
    std::mem::transmute(*include_bytes!("../../nn-7ea3423c81571dd7.nnue"))
};

const FT_BIAS_OFFSET: usize = 16 * 6 * 64;
const LAYER_1_WEIGHTS: usize = FT_BIAS_OFFSET + 32;
const LAYER_1_BIAS: usize = LAYER_1_WEIGHTS + 4 * 32;
const LAYER_2_WEIGHTS: usize = LAYER_1_BIAS + 4;
const LAYER_2_BIAS: usize = LAYER_2_WEIGHTS + 4 * 4;
const LAYER_3_WEIGHTS: usize = LAYER_2_BIAS + 4;
const LAYER_3_BIAS: usize = LAYER_3_WEIGHTS + 4;

const EVAL_SCALE: f32 = 256.0 / 0.75;

const MATERIAL: [i16; 5] = [
    288,
    730,
    771,
    1296,
    2378,
];

fn apply_ft(pieces: &[Bitboard; 6], mask: u32) -> (__m256, __m256) {
    unsafe {
        let mut v0 = _mm256_setzero_ps();
        let mut v1 = _mm256_setzero_ps();

        for (i, mut piece) in pieces.iter().copied().enumerate() {
            while piece != 0 {
                let square = piece.trailing_zeros() ^ mask;
                let index = 16 * (i * 64 + square as usize);
                v0 = _mm256_add_ps(v0, _mm256_loadu_ps(NN.as_ptr().add(index)));
                v1 = _mm256_add_ps(v1, _mm256_loadu_ps(NN.as_ptr().add(index + 8)));

                piece &= piece - 1;
            }
        }

        (v0, v1)
    }
}

pub fn evaluate(board: &Board) -> i32 {
    let material = {
        let mut material = 0;
        for (i, weight) in MATERIAL.into_iter().enumerate() {
            let count = board.pieces()[0][i].count_ones() as i16 - board.pieces()[1][i].count_ones() as i16;
            material += weight * count;
        }

        if board.side_to_move() == Color::White {
            material as i32
        } else {
            -material as i32
        }
    };

    let (v0, v1, v2, v3) = if board.side_to_move() == Color::White {
        let (v0, v1) = apply_ft(&board.pieces()[0], 0);
        let (v2, v3) = apply_ft(&board.pieces()[1], 56);
        (v0, v1, v2, v3)
    } else {
        let (v0, v1) = apply_ft(&board.pieces()[1], 56);
        let (v2, v3) = apply_ft(&board.pieces()[0], 0);
        (v0, v1, v2, v3)
    };

    unsafe {
        let v0 = _mm256_add_ps(v0, _mm256_loadu_ps(NN.as_ptr().add(FT_BIAS_OFFSET)));
        let v1 = _mm256_add_ps(v1, _mm256_loadu_ps(NN.as_ptr().add(FT_BIAS_OFFSET + 8)));
        let v2 = _mm256_add_ps(v2, _mm256_loadu_ps(NN.as_ptr().add(FT_BIAS_OFFSET + 16)));
        let v3 = _mm256_add_ps(v3, _mm256_loadu_ps(NN.as_ptr().add(FT_BIAS_OFFSET + 24)));

        // relu
        let v0 = _mm256_max_ps(v0, _mm256_setzero_ps());
        let v1 = _mm256_max_ps(v1, _mm256_setzero_ps());
        let v2 = _mm256_max_ps(v2, _mm256_setzero_ps());
        let v3 = _mm256_max_ps(v3, _mm256_setzero_ps());


        let mut a0 = _mm256_setzero_ps();
        let mut a1 = _mm256_setzero_ps();
        let mut a2 = _mm256_setzero_ps();
        let mut a3 = _mm256_setzero_ps();

        let mut perm = _mm256_set1_epi32(0b11100100);

        // let v0
        for i in 0..4 {
            a0 = _mm256_fmadd_ps(
                _mm256_permutevar_ps(v0, perm),
                _mm256_loadu_ps(NN.as_ptr().add(LAYER_1_WEIGHTS + i * 32)),
                a0,
            );

            a1 = _mm256_fmadd_ps(
                _mm256_permutevar_ps(v1, perm),
                _mm256_loadu_ps(NN.as_ptr().add(LAYER_1_WEIGHTS + i * 32 + 8)),
                a1,
            );

            a2 = _mm256_fmadd_ps(
                _mm256_permutevar_ps(v2, perm),
                _mm256_loadu_ps(NN.as_ptr().add(LAYER_1_WEIGHTS + i * 32 + 16)),
                a2,
            );

            a3 = _mm256_fmadd_ps(
                _mm256_permutevar_ps(v3, perm),
                _mm256_loadu_ps(NN.as_ptr().add(LAYER_1_WEIGHTS + i * 32 + 24)),
                a3,
            );

            perm = _mm256_srli_epi32::<2>(perm);
        }

        let a0 = _mm256_add_ps(a0, a1);
        let a1 = _mm256_add_ps(a2, a3);
        let acc = _mm256_add_ps(a0, a1);
        let acc = _mm_add_ps(
            _mm256_castps256_ps128(acc),
            _mm256_extractf128_ps(acc, 1)
        );

        let acc = _mm_add_ps(acc, _mm_loadu_ps(NN.as_ptr().add(LAYER_1_BIAS)));
        let acc = _mm_max_ps(acc, _mm_setzero_ps());

        let a0 = _mm_dp_ps::<0b11110001>(acc, _mm_loadu_ps(NN.as_ptr().add(LAYER_2_WEIGHTS)));
        let a1 = _mm_dp_ps::<0b11110010>(acc, _mm_loadu_ps(NN.as_ptr().add(LAYER_2_WEIGHTS + 4)));
        let a2 = _mm_dp_ps::<0b11110100>(acc, _mm_loadu_ps(NN.as_ptr().add(LAYER_2_WEIGHTS + 8)));
        let a3 = _mm_dp_ps::<0b11111000>(acc, _mm_loadu_ps(NN.as_ptr().add(LAYER_2_WEIGHTS + 12)));

        let acc = _mm_or_ps(
            _mm_or_ps(a0, a1),
            _mm_or_ps(a2, a3),
        );

    
        let acc = _mm_add_ps(acc, _mm_loadu_ps(NN.as_ptr().add(LAYER_2_BIAS)));
        let acc = _mm_max_ps(acc, _mm_setzero_ps());

        let acc = _mm_dp_ps::<0b11110001>(acc, _mm_loadu_ps(NN.as_ptr().add(LAYER_3_WEIGHTS)));
        let eval = _mm_cvtss_f32(acc);

        (((eval + NN[LAYER_3_BIAS]) * EVAL_SCALE) as i32 + material)
            .clamp(-64 * 256, 64 * 256) // just in case
    }
}
