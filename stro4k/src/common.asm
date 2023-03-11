NUM_THREADS equ 4
MAX_BOARDS equ 6144
THREAD_STACK_SIZE equ 4 * 1024 * 1024


%ifdef EXPORT_SYSV
global SHIFTS
%endif

; TODO align 64 bytes and take advantage in addressing
struc Board
    alignb 8
    .pieces:
    .white_pieces:
        resq 6
    .black_pieces:
        resq 6
    .colors:
    .white:
        resq 1
    .black:
        resq 1
    .side_to_move:
        resb 1
    .ep:
        resb 1
    .castling:
        resb 1
    .fifty_moves:
        resb 1
    .padding:
        resb 12
endstruc

%if Board_size != 128
%error "Board should be 128 bytes in size"
%endif

DOUBLE_PAWN_PUSH_FLAG equ 0001b
QUEENSIDE_CASTLE_FLAG equ 0010b
KINGSIDE_CASTLE_FLAG equ 0011b
CAPTURE_FLAG equ 0100b
PROMO_FLAG equ 1000b
EN_PASSANT_FLAG equ CAPTURE_FLAG | 0001b

struc PlyData
    alignb 8
    .kt:
        resw 2
    .no_nmp:
        resb 1
endstruc

struc Search
    .game:
        resq 1
    .nodes:
        resq 1
    .start_tvsec:
        resq 1
    .start_tvnsec:
        resq 1
    .search_time:
        resq 1
    .tt:
        resq 1
    .running:
        resq 1
    .history:
    .white_history:
        resq 64 * 64
    .black_history:
        resq 64 * 64
        alignb 8
    .ply_data:
        resb PlyData_size * MAX_BOARDS
endstruc

READ_SYSCALL equ 0
WRITE_SYSCALL equ 1
MMAP_SYSCALL equ 9
EXIT_SYSCALL equ 60
CLOCK_GETTIME_SYSCALL equ 228

PROT_READ equ 1
PROT_WRITE equ 2
MAP_PRIVATE equ 2
MAP_ANONYMOUS equ 20h

section .rodata
alignb 8
ALL_MASK:
    dq 0FFFF_FFFF_FFFF_FFFFh
NOT_A_FILE:
    dq ~0101_0101_0101_0101h
NOT_H_FILE:
    dq ~8080_8080_8080_8080h
NOT_AB_FILE:
    dq ~0303_0303_0303_0303h
NOT_GH_FILE:
    dq ~0C0C0_C0C0_C0C0_C0C0h

section .bss
alignb 8
SHIFTS:
ROOK_SHIFTS:
    resq 2
BISHOP_SHIFTS:
    resq 2
KNIGHT_SHIFTS:
    resq 4

