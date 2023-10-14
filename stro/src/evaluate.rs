use crate::position::{Bitboard, Board, Color};
use std::arch::x86_64::*;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct Eval(i16, i16);

pub const MAX_EVAL: i32 = 128 * 256 - 1;
pub const MIN_EVAL: i32 = -MAX_EVAL;

#[cfg(feature = "nn_path_env")]
const NN_DATA: [f32; 6454] = unsafe {
    std::mem::transmute(*include_bytes!(concat!("../", env!("NN_PATH"))))
};

#[cfg(not(feature = "nn_path_env"))]
const NN_DATA: [f32; 6454] = unsafe {
    std::mem::transmute(*include_bytes!("../../nn-e3a9863b410c4598.nnue"))
};

macro_rules! unsafe_nn_features {
    (
        $data:ident,
        const $feature:ident: &$type_:ty;
        $($tail:tt)*
    ) => {
        unsafe_nn_features! {
            @inner
            $data,
            0,
            const $feature: &$type_;
            $($tail)*
        }
    };
    (
        @inner
        $data:ident,
        $offset:expr,
        const $feature:ident: &$type_:ty;
        $($tail:tt)*
    ) => {
        const $feature: &$type_ = unsafe {
            &*(($data.as_ptr() as *const ::core::primitive::u8).add($offset) as *const _)
        };

        unsafe_nn_features! {
            @inner
            $data,
            $offset + ::core::mem::size_of::<$type_>(),
            $($tail)*
        }
    };
    (
        @inner
        $data:ident,
        $offset:expr,
    ) => {
        fn _size_check() {
            unsafe {
                ::core::mem::transmute::<_, [::core::primitive::u8; $offset]>($data);
            }
        }
    };
}

unsafe_nn_features! {
    NN_DATA,
    const MATERIAL: &[i32; 5];
    const FT_WEIGHTS: &[f32; 16 * 6 * 64];
    const FT_BIAS: &[f32; 32];
    const LAYER_1_WEIGHTS: &[f32; 32 * 8];
    const LAYER_1_BIAS: &[f32; 8];
    const LAYER_2_WEIGHTS: &[f32; 8];
    const LAYER_2_BIAS: &f32;
}

fn apply_ft(pieces: &[Bitboard; 6], mask: u32) -> (__m256, __m256) {
    unsafe {
        let mut v0 = _mm256_setzero_ps();
        let mut v1 = _mm256_setzero_ps();

        for (i, mut piece) in pieces.iter().copied().enumerate() {
            while piece != 0 {
                let square = piece.trailing_zeros() ^ mask;
                let index = 16 * (i * 64 + square as usize);
                v0 = _mm256_add_ps(v0, _mm256_loadu_ps(FT_WEIGHTS.as_ptr().add(index)));
                v1 = _mm256_add_ps(v1, _mm256_loadu_ps(FT_WEIGHTS.as_ptr().add(index + 8)));

                piece &= piece - 1;
            }
        }

        (v0, v1)
    }
}

pub fn evaluate(board: &Board) -> i32 {
    let material = {
        let mut material = 0;
        for (i, value) in MATERIAL.iter().enumerate() {
            let count = board.pieces()[0][i].count_ones() as i32
                - board.pieces()[1][i].count_ones() as i32;

            material += count * value;
        }

        material
    };

    let (v0, v1, v2, v3, material) = if board.side_to_move() == Color::White {
        let (v0, v1) = apply_ft(&board.pieces()[0], 0);
        let (v2, v3) = apply_ft(&board.pieces()[1], 56);
        (v0, v1, v2, v3, material)
    } else {
        let (v0, v1) = apply_ft(&board.pieces()[1], 56);
        let (v2, v3) = apply_ft(&board.pieces()[0], 0);
        (v0, v1, v2, v3, -material)
    };

    unsafe {
        let v0 = _mm256_add_ps(v0, _mm256_loadu_ps(FT_BIAS.as_ptr().add(0)));
        let v1 = _mm256_add_ps(v1, _mm256_loadu_ps(FT_BIAS.as_ptr().add(8)));
        let v2 = _mm256_add_ps(v2, _mm256_loadu_ps(FT_BIAS.as_ptr().add(16)));
        let v3 = _mm256_add_ps(v3, _mm256_loadu_ps(FT_BIAS.as_ptr().add(24)));

        // relu
        let v0 = _mm256_max_ps(v0, _mm256_setzero_ps());
        let v1 = _mm256_max_ps(v1, _mm256_setzero_ps());
        let v2 = _mm256_max_ps(v2, _mm256_setzero_ps());
        let v3 = _mm256_max_ps(v3, _mm256_setzero_ps());


        // Layer 1
        let mut a0 = _mm256_setzero_ps();
        let mut a1 = _mm256_setzero_ps();
        let mut a2 = _mm256_setzero_ps();
        let mut a3 = _mm256_setzero_ps();

        let mut perm = _mm256_set1_epi32(0o76543210);

        for i in 0..8 {
            a0 = _mm256_fmadd_ps(
                _mm256_permutevar8x32_ps(v0, perm),
                _mm256_loadu_ps(LAYER_1_WEIGHTS.as_ptr().add(i * 32)),
                a0,
            );

            a1 = _mm256_fmadd_ps(
                _mm256_permutevar8x32_ps(v1, perm),
                _mm256_loadu_ps(LAYER_1_WEIGHTS.as_ptr().add(i * 32 + 8)),
                a1,
            );

            a2 = _mm256_fmadd_ps(
                _mm256_permutevar8x32_ps(v2, perm),
                _mm256_loadu_ps(LAYER_1_WEIGHTS.as_ptr().add(i * 32 + 16)),
                a2,
            );

            a3 = _mm256_fmadd_ps(
                _mm256_permutevar8x32_ps(v3, perm),
                _mm256_loadu_ps(LAYER_1_WEIGHTS.as_ptr().add(i * 32 + 24)),
                a3,
            );

            perm = _mm256_srli_epi32::<3>(perm);
        }

        let a0 = _mm256_add_ps(a0, a1);
        let a1 = _mm256_add_ps(a2, a3);
        let acc = _mm256_add_ps(a0, a1);

        let acc = _mm256_add_ps(acc, _mm256_loadu_ps(LAYER_1_BIAS.as_ptr()));

        // Relu
        let acc = _mm256_max_ps(acc, _mm256_setzero_ps());

        // Layer 2
        let acc = _mm256_dp_ps::<0b11110001>(acc, _mm256_loadu_ps(LAYER_2_WEIGHTS.as_ptr()));
        let acc = _mm_add_ss(
            _mm256_castps256_ps128(acc),
            _mm256_extractf128_ps(acc, 1),
        );

        ((_mm_cvtss_f32(acc) + LAYER_2_BIAS) as i32 + material)
            .clamp(-64 * 256, 64 * 256) // just in case
    }
}
