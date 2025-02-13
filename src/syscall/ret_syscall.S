.global ret_from_syscall
ret_from_syscall:
    pop r11
    pop rcx
    sysretq
