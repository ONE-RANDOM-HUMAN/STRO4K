%ifndef NUM_THREADS
%define NUM_THREADS 4
%endif

MAX_BOARDS equ 6144
THREAD_STACK_SIZE equ 8 * 1024 * 1024

%ifndef EXPORT_SYSV
%ifndef TT_SIZE_MB
%define TT_SIZE_MB 16
%endif
TT_SIZE_BYTES equ TT_SIZE_MB * 1024 * 1024
TT_ENTRY_COUNT equ TT_SIZE_BYTES / 8

%if TT_ENTRY_COUNT & (TT_ENTRY_COUNT - 1) != 0
%error "TT entry count must be a power of 2"
%endif

%else
global SHIFTS
extern TT_PTR
extern TT_MASK
extern RUNNING
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
    .static_eval:
        resw 1
    .no_nmp:
        resb 1
    alignb 8
endstruc


%if PlyData_size != 8
%error "PlyData should be 8 bytes in size"
%endif

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
    .min_search_time:
        resq 1
    .max_search_time:
        resq 1
    alignb 16
    .ply_data:
        resb PlyData_size * MAX_BOARDS
    .white_history:
        resq 64 * 64
    .black_history:
        resq 64 * 64
endstruc

%if Search_size % 16 != 0
%error "Search should be a multiple of 16 bytes in size"
%endif

struc SearchMove
    alignb 4
    .score:
        resw 1
    .move:
        resw 1
endstruc

%if SearchMove_size != 4
%error "SearchMove should be 4 bytes in size"
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

LIGHT_SQUARES equ 55AA55AA55AA55AAh

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

%ifndef EXPORT_SYSV
RUNNING_WORKER_THREADS:
    ; the top bit will indicate whether the threads should continue running
    resb 1
%endif

alignb 4096
THREAD_STACKS:
    times NUM_THREADS resb THREAD_STACK_SIZE

%ifndef EXPORT_SYSV
TT_MEM:
    resb TT_SIZE_BYTES
%endif

