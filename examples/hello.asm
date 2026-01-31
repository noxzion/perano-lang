.NVM0

; Function: main
fn_main:
    enter 1
    ; call novaria.Open
    ; Inline novaria.Open
    push 47
    push 101
    push 116
    push 99
    push 47
    push 111
    push 115
    push 95
    push 114
    push 101
    push 108
    push 101
    push 97
    push 115
    push 101
    push 0
    syscall 2
    ; Main returns 0 by default
    push 10
    syscall print
    push 0
    syscall exit
    leave
    ret

; Module Function: novaria_Open
fn_novaria_Open:
    ; param: path - arg 0
    ; var fd int
    push 0
    storer 0
    ; inline asm
    syscall 2
    storer 0
    loadr 0  ; local fd
    leave
    ret
    ret

