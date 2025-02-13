.global ret_from_syscall
ret_from_syscall:
    pop rcx
    pop r11
    sysretq
