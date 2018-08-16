BITS 64
;; Preamble
mov rax, rdi
;; Add
add rax, rdi
;; Sub
sub rax, rdi
;; Mul
imul rax, rdi
;; Div
xor rdx, rdx
idiv rdi
;; Mod = Div + below
mov rax, rdx
;; Xor
xor rax, rdi
;; Shl/Shr Preamble
mov ecx, edi
;; Shl
shl rax, cl
;; Shr
shr rax, cl
;; And
and rax, rdi
;; Or
or rax, rdi
;; Not
not rax
;; Epilog
ret
