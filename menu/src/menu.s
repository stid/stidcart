; stidcart menu: lists a cart's entries and starts them.
; GPL-2.0-or-later, see COPYING at the repository root.
;
; Loaded at bank 0 ROML ($8000). The boot code enters through the first word in 16K mode, bank 0,
; with the KERNAL not initialised (docs/format/easyflash-cart-format.md, "How the boot code hands
; over to the menu loader"). Bank 0 ROMH, at $A000 here, holds the EasyFS2 directory and the cart
; name.
;
; Keys are read only through GETIN, i.e. the KERNAL keyboard buffer, which is also the only place
; VICE can inject them (D-038). menu_idle is reached only when that buffer is empty: the boot tests
; break there before pressing anything.

CHROUT  = $ffd2
GETIN   = $ffe4

BANK    = $de00                 ; EasyFlash bank register
CONTROL = $de02                 ; EasyFlash mode register
MODE_OFF = $04
MODE_ULT = $05
MODE_8K  = $06
MODE_16K = $07

DIR     = $a000                 ; EasyFS2 directory, 24-byte slots
EFNAME  = $bb08                 ; cart name, 16 bytes PETSCII, NUL padded
SLOT    = 24
ROWS    = 20                    ; entries shown at once

KEY_DOWN   = $11
KEY_UP     = $91
KEY_RETURN = $0d
KEY_STOP   = $03

ptr     = $fb                   ; 2 bytes: a directory slot, later the copy source
dst     = $fd                   ; 2 bytes: the copy destination
; make_keys runs after the copy, when $C000 may hold the program, so it keeps to zero page:
num     = $fb                   ; 2 bytes, the SYS number (ptr is free by then)
digit   = $fd                   ; (dst is free by then)
lead    = $fe
tmp     = $02

; The menu's own variables. Nothing here outlives the menu: a program it starts may load over them.
visible = $c000                 ; slot numbers of the visible entries, in directory order
count   = $c100                 ; how many are visible
sel     = $c101                 ; the selected one, an index into visible
top     = $c102                 ; the first one on screen
row     = $c103
cur     = $c104

        * = $8000
        .word entry

entry:
        sei
        ldx #0
-       lda ram_rom,x
        sta ram_start,x
        inx
        cpx #ram_end - ram_start
        bne -
        jmp kernal_init         ; comes back to back_from_init with the KERNAL initialised

back_from_init:
        cli
        lda #0
        sta sel
        sta top
        jsr scan
        jsr draw

loop:
        jsr GETIN
        bne key
menu_idle:
        jmp loop

key:
        cmp #KEY_DOWN
        beq down
        cmp #KEY_UP
        beq up
        cmp #KEY_RETURN
        beq start
        cmp #KEY_STOP
        bne loop
        jmp leave

down:
        ldx sel
        inx
        cpx count
        bcs loop                ; already on the last entry
        stx sel
        txa
        sec
        sbc top
        cmp #ROWS
        bcc redraw
        inc top
redraw:
        jsr draw
        jmp loop

up:
        lda sel
        beq loop
        dec sel
        lda sel
        cmp top
        bcs redraw
        sta top
        jmp redraw

; --- starting the selected entry --------------------------------------------------------------

start:
        lda count
        beq loop
        ldx sel
        lda visible,x
        jsr slot_ptr
        ldy #16
        lda (ptr),y
        and #$1f
        sta cur                 ; entry type
        ldy #17
        lda (ptr),y
        sta p_bank
        iny
        lda (ptr),y
        sta p_bank+1
        lda cur
        cmp #$04
        bcc start_prg           ; $01-$03: a program

        ldx #MODE_8K
        cmp #$10
        beq _cart
        cmp #$1c
        beq _cart
        ldx #MODE_16K
        cmp #$11
        beq _cart
        cmp #$1d
        beq _cart
        ldx #MODE_ULT
        cmp #$12
        beq _cart
        cmp #$13
        beq _cart
        cmp #$1e
        beq _cart
        jmp loop                ; nothing else is startable by this menu
_cart:
        stx cur                 ; the mode
        sei
        lda #$7f                ; quiet the machine as a power-on leaves it
        sta $dc0d
        sta $dd0d
        lda $dc0d
        lda $dd0d
        lda #0
        ldx #$2e
-       sta $d000,x
        dex
        bpl -
        lda p_bank              ; where an xbank cart looks for its first bank
        sta $df00
        lda p_bank+1
        sta $df01
        ldx p_bank
        ldy cur
        jmp cart_go

; A program: bytes 19-20 are its offset in the bank, bit 13 selecting ROMH, so in 16K mode its
; first byte is at $8000 + (offset & $3fff). Types $01-$03 differ only in where a bank runs out.
start_prg:
        ldy #19
        lda (ptr),y
        sta p_src
        iny
        lda (ptr),y
        and #$3f
        ora #$80
        sta p_src+1
        ldx cur
        lda wrap_start-1,x
        sta p_wstart
        lda wrap_end-1,x
        sta p_wend
        ldy #21
        lda (ptr),y
        sta p_size
        iny
        lda (ptr),y
        sta p_size+1
        iny
        lda (ptr),y
        sta p_size+2
        jsr read_load
        sec                     ; the load address is not copied
        lda p_size
        sbc #2
        sta p_size
        lda p_size+1
        sbc #0
        sta p_size+1
        lda p_size+2
        sbc #0
        sta p_size+2
        lda $0324               ; BASIC's first CHRIN goes to the trap
        sta p_chrin
        lda $0325
        sta p_chrin+1
        lda #<trap
        sta $0324
        lda #>trap
        sta $0325
        jmp basic_go

; Back from the trap with the program copied, the cartridge on and dst one past its last byte:
; set BASIC's pointers to its end, type RUN or SYS for it, and leave through the real CHRIN.
copied:
        lda #$01
        sta $2b
        lda #$08
        sta $2c
        lda dst
        sta $2d
        sta $2f
        sta $31
        sta $ae
        lda dst+1
        sta $2e
        sta $30
        sta $32
        sta $af
        lda #8
        sta $ba                 ; last device used: the drive, as after a LOAD
        jsr make_keys
        stx $c6
        jmp basic_resume

; Where the copy source wraps to the next bank, by type: $01 interleaved ROML then ROMH, $02 ROML
; only, $03 ROMH only (format spec, "Bytes 19-20 for PRG entries").
wrap_start:
        .byte $80, $80, $a0
wrap_end:
        .byte $c0, $a0, $c0

; Put what BASIC is to type in the keyboard buffer, returning its length in X: RUN for a load
; address at or below $0801, otherwise SYS and the load address.
make_keys:
        lda p_load+1
        cmp #$08
        bcc _run
        bne _sys
        lda p_load
        cmp #$02
        bcc _run
_sys:
        ldx #0
-       lda sys_text,x
        sta $0277,x
        inx
        cpx #4
        bne -
        lda p_load
        sta num
        lda p_load+1
        sta num+1
        lda #0
        sta lead
        ldy #0
_pow:
        lda #"0"
        sta digit
_sub:
        lda num
        sec
        sbc pow_lo,y
        sta tmp
        lda num+1
        sbc pow_hi,y
        bcc _emit
        sta num+1
        lda tmp
        sta num
        inc digit
        jmp _sub
_emit:
        lda digit
        cmp #"0"
        bne _put
        lda lead
        bne _put
        cpy #4
        bne _skip               ; a leading zero, and not the last digit
_put:
        lda digit
        sta $0277,x
        inx
        lda #1
        sta lead
_skip:
        iny
        cpy #5
        bne _pow
        lda #KEY_RETURN
        sta $0277,x
        inx
        rts
_run:
        ldx #0
-       lda run_text,x
        sta $0277,x
        inx
        cpx #4
        bne -
        rts

sys_text:
        .text "sys "
run_text:
        .text "run", KEY_RETURN
pow_lo:
        .byte <10000, <1000, <100, <10, <1
pow_hi:
        .byte >10000, >1000, >100, >10, >1

; --- the listing ------------------------------------------------------------------------------

; Collect the visible entries' slot numbers, stopping at the end marker.
scan:
        lda #<DIR
        sta ptr
        lda #>DIR
        sta ptr+1
        lda #0
        sta count
        ldx #0
_next:
        ldy #16
        lda (ptr),y
        and #$1f
        cmp #$1f
        beq _done               ; end marker
        lda (ptr),y
        bmi _skip               ; hidden
        ldy count
        txa
        sta visible,y
        inc count
_skip:
        clc
        lda ptr
        adc #SLOT
        sta ptr
        bcc +
        inc ptr+1
+       inx
        bne _next
_done:
        rts

; ptr = DIR + A * 24
slot_ptr:
        ldx #0
        stx ptr+1
        asl a
        rol ptr+1
        asl a
        rol ptr+1
        asl a
        rol ptr+1               ; ptr+1:A = slot * 8
        sta ptr
        ldx ptr+1
        asl a
        rol ptr+1               ; ptr+1:A = slot * 16
        clc
        adc ptr
        sta ptr
        txa
        adc ptr+1
        sta ptr+1               ; ptr = slot * 24
        clc
        lda ptr+1
        adc #>DIR
        sta ptr+1
        rts

draw:
        lda #$93                ; clear the screen
        jsr CHROUT
        ldx #0
-       lda EFNAME,x
        beq +
        jsr putc
        inx
        cpx #16
        bne -
+       lda #KEY_RETURN
        jsr CHROUT
        jsr CHROUT
        lda count
        bne _rows
        ldx #0
-       lda no_entries,x
        beq _done
        jsr CHROUT
        inx
        bne -
_rows:
        lda #0
        sta row
_row:
        lda row
        clc
        adc top
        cmp count
        bcs _done               ; past the last entry
        sta cur
        cmp sel
        bne +
        lda #$12                ; reverse on: the selected entry
        jsr CHROUT
+       ldx cur
        lda visible,x
        jsr slot_ptr
        ldy #0
-       lda (ptr),y
        beq +
        jsr putc
        iny
        cpy #16
        bne -
+       lda #$92                ; reverse off
        jsr CHROUT
        lda #KEY_RETURN
        jsr CHROUT
        inc row
        lda row
        cmp #ROWS
        bcc _row
_done:
        rts

; Print a name byte, as "?" when CHROUT would take it as a control code.
putc:
        cmp #$20
        bcc _bad
        cmp #$80
        bcc _ok
        cmp #$a0
        bcs _ok
_bad:
        lda #"?"
_ok:
        jmp CHROUT

no_entries:
        .null "no entries"

        .include "launch.s"

        .cerror * > $a000, "the menu outgrows the 8 KiB loader slot"
        .fill $a000 - *, $ff
