NUM_THREADS equ 4
MAX_BOARDS equ 6144
THREAD_STACK_SIZE equ 8 * 1024 * 1024
TT_SIZE_BYTES equ 16 * 1024 * 1024

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
    .start_time:
    .start_tvsec:
        resq 1
    .start_tvnsec:
        resq 1
    .search_time:
        resq 1
    .history:
    alignb 16
    .white_history:
        resq 64 * 64
    .black_history:
        resq 64 * 64
    .ply_data:
        resb PlyData_size * MAX_BOARDS
endstruc

%if Search_size % 16 != 0
%error "Search should be a multiple of 16 bytes in size"
%endif

READ_SYSCALL equ 0
WRITE_SYSCALL equ 1
MMAP_SYSCALL equ 9
CLONE_SYSCALL equ 56
EXIT_SYSCALL equ 60
CLOCK_GETTIME_SYSCALL equ 228

PROT_READ equ 1
PROT_WRITE equ 2
MAP_PRIVATE equ 2
MAP_ANONYMOUS equ 20h

CLONE_VM equ 00000100h
CLONE_FS equ 00000200h
CLONE_FILES equ 00000400h
CLONE_SIGHAND equ 00000800h
CLONE_THREAD equ 00010000h

CLOCK_MONOTONIC equ 1

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

; TODO: reduce size of this
STARTPOS:
    dq 0x0000_0000_0000_FF00
    dq 0x0000_0000_0000_0042
    dq 0x0000_0000_0000_0024
    dq 0x0000_0000_0000_0081
    dq 0x0000_0000_0000_0008
    dq 0x0000_0000_0000_0010

    dq 0x00FF_0000_0000_0000
    dq 0x4200_0000_0000_0000
    dq 0x2400_0000_0000_0000
    dq 0x8100_0000_0000_0000
    dq 0x0800_0000_0000_0000
    dq 0x1000_0000_0000_0000

    dq 0x0000_0000_0000_FFFF
    dq 0xFFFF_0000_0000_0000
    dd 000F4000h

section .bss
alignb 8
SHIFTS:
ROOK_SHIFTS:
    resq 2
BISHOP_SHIFTS:
    resq 2
KNIGHT_SHIFTS:
    resq 4

RUNNING_WORKER_THREADS:
    ; the top bit will indicate whether the threads should continue running
    resb 1

alignb 4096
THREAD_STACKS:
    times NUM_THREADS resb THREAD_STACK_SIZE

TT_MEM:
    resb TT_SIZE_BYTES

