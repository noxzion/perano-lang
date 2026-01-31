.NVM0

; Function: main
fn_main:
    enter 1
    ; call stdio.Println
    push 4
    push 2
    div
    storer 0
    loadr 0
    call __print_int_sys
    push '\n'
    syscall print
    ; Main returns 0 by default
    push 10
    syscall print
    push 0
    syscall exit
    leave
    ret

__print_int_sys:
    enter 3
    loada 0
    storer 0
    loadr 0
    push 0
    eq
    jz __pint_zero_cont
__pint_zero:
    push '0'
    syscall print
    leave
    ret
__pint_zero_cont:
    loadr 0
    push 0
    lt
    jz __pint_not_neg
    push '-'
    syscall print
    loadr 0
    push 0
    swap
    sub
    storer 0
__pint_not_neg:
    push 1
    storer 1
__pint_find:
    loadr 1
    push 10
    mul
    loadr 0
    gt
    jnz __pint_find_done
    loadr 1
    push 10
    mul
    storer 1
    jmp __pint_find
__pint_find_done:
__pint_loop:
    loadr 1
    push 0
    gt
    jz __pint_done
    loadr 0
    loadr 1
    div
    storer 2
    loadr 2
    push '0'
    add
    syscall print
    loadr 0
    loadr 1
    mod
    storer 0
    loadr 1
    push 10
    div
    storer 1
    jmp __pint_loop
__pint_done:
    leave
    ret
