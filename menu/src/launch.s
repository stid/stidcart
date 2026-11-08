; stidcart menu: the routines that run from cartridge RAM at $DF02, where the menu and the KERNAL
; can be switched out from under them.
; GPL-2.0-or-later, see COPYING at the repository root.
;
; The sequences follow EasyLoader's technique (ndefpack src/fw/easyloader.asm:1789, 2091-2260),
; written fresh: a partial reset with the cartridge off, and a program copied only after BASIC's
; cold start has run, from a trap on its first CHRIN, so nothing the init touches can overwrite the
; program (D-038). $DF00/$DF01 stay free: they carry an xbank entry's first bank.

ram_rom:
        .logical $df02
ram_start:

; Initialise the KERNAL with the cartridge off, so RAMTAS finds the real top of memory, then put
; the cartridge back and return to the menu. The stack is reset, so this returns by jumping.
kernal_init:
        lda #MODE_OFF
        sta CONTROL
        ldx #$ff
        txs
        jsr $fda3               ; IOINIT
        jsr $fd50               ; RAMTAS
        jsr $fd15               ; RESTOR
        jsr $ff5b               ; CINT
        lda #MODE_16K
        sta CONTROL
        jmp back_from_init

; RUN/STOP: switch the cartridge off and reset into BASIC.
leave:
        lda #MODE_OFF
        sta CONTROL
        jmp ($fffc)

; Start a cartridge entry: X its first bank, Y its mode. The menu has already quieted the machine
; and left the bank in $DF00/$DF01 for an xbank cart.
cart_go:
        stx BANK
        sty CONTROL
        jmp ($fffc)

; Hand the machine to BASIC, whose first CHRIN the menu has routed to the trap.
basic_go:
        lda #MODE_OFF
        sta CONTROL
        jmp $fcfe               ; CLI, then BASIC's cold start through ($A000)

; Point the copy source at the entry's first byte.
src_init:
        lda p_src
        sta ptr
        lda p_src+1
        sta ptr+1
        lda p_bank
        sta p_cbank
        sta BANK
        rts

; The entry's next byte in A, moving to the next bank where this type runs out of one.
get_byte:
        ldy #0
        lda (ptr),y
        inc ptr
        bne _done
        inc ptr+1
        pha
        lda ptr+1
        cmp p_wend
        bne _same
        lda p_wstart
        sta ptr+1
        inc p_cbank
        lda p_cbank
        sta BANK
_same:
        pla
_done:
        rts

; The program's load address, read for the menu, which then needs bank 0 back.
read_load:
        jsr src_init
        jsr get_byte
        sta p_load
        jsr get_byte
        sta p_load+1
        lda #0
        sta BANK
        rts

; BASIC is up and asks for its first line: restore CHRIN, copy the program in with the cartridge
; on, then let the menu finish from ROM (copied). The caller's registers are kept for the real
; CHRIN: BASIC's line input holds its buffer index in X across the call.
trap:
        pha
        txa
        pha
        tya
        pha
        lda p_chrin
        sta $0324
        lda p_chrin+1
        sta $0325
        lda #MODE_16K
        sta CONTROL
        jsr src_init
        jsr get_byte
        sta dst
        jsr get_byte
        sta dst+1
_copy:
        lda p_size
        ora p_size+1
        ora p_size+2
        beq _copied
        jsr get_byte
        ldy #0
        sta (dst),y
        inc dst
        bne _count
        inc dst+1
_count:
        lda p_size
        bne _d0
        lda p_size+1
        bne _d1
        dec p_size+2
_d1:
        dec p_size+1
_d0:
        dec p_size
        jmp _copy
_copied:
        lda #0
        sta BANK
        jmp copied

; The program is in place and BASIC is about to read what the menu typed for it.
basic_resume:
        lda #MODE_OFF
        sta CONTROL
        pla
        tay
        pla
        tax
        pla
        jmp (p_chrin)

; Parameters the menu fills in before handing over.
p_bank:   .word 0               ; the entry's first bank
p_src:    .word 0               ; first byte's address with the bank selected
p_wstart: .byte 0               ; high byte the source wraps to
p_wend:   .byte 0               ; high byte at which it wraps
p_size:   .byte 0, 0, 0         ; bytes still to copy
p_cbank:  .byte 0
p_load:   .word 0
p_chrin:  .word 0
p_only:   .byte 0               ; non-zero: load the program but type nothing (SHIFT+RETURN)

ram_end:
        .here

        .cerror ram_end - ram_start > $fe, "the RAM routines outgrow $DF02-$DFFF"
