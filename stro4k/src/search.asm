default rel
section .text

thread_search:
    push rsp
    pop rdi
    xor ecx, ecx
    call root_search_sysv

    push EXIT_SYSCALL
    pop rax
    xor edi, edi

    lock dec byte [RUNNING_WORKER_THREADS]

    syscall
