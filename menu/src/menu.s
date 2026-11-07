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
;
; Typed characters filter the list to the entries whose name contains them, and F1 switches between
; directory order and A-Z (D-039). Both compare letters through fold, so case never matters.

CHROUT  = $ffd2
GETIN   = $ffe4
PLOT    = $fff0

BANK    = $de00                 ; EasyFlash bank register
CONTROL = $de02                 ; EasyFlash mode register
MODE_OFF = $04
MODE_ULT = $05
MODE_8K  = $06
MODE_16K = $07

DIR     = $a000                 ; EasyFS2 directory, 24-byte slots
EFNAME  = $bb08                 ; cart name, 16 bytes PETSCII, NUL padded
SLOT    = 24
ROWS    = 20                    ; entries shown at once, on screen rows 2-21
STATUS  = 23                    ; the screen row with the count and the order
QMAX    = 16                    ; the longest text the filter takes, a whole name

KEY_DOWN   = $11
KEY_UP     = $91
KEY_RETURN = $0d
KEY_STOP   = $03
KEY_DEL    = $14
KEY_HOME   = $13
KEY_CLR    = $93
KEY_F1     = $85

ptr     = $fb                   ; 2 bytes: a directory slot, later the copy source
dst     = $fd                   ; 2 bytes: the copy destination; in the menu, a second pointer
; make_keys runs after the copy, when $C000 may hold the program, so it keeps to zero page:
num     = $fb                   ; 2 bytes, the SYS number (ptr is free by then)
digit   = $fd                   ; (dst is free by then)
lead    = $fe
tmp     = $02

; The menu's own variables. Nothing here outlives the menu: a program it starts may load over them.
visible = $c000                 ; slot numbers of the visible entries, in directory order
count   = $c100                 ; how many are visible
sel     = $c101                 ; the selected one, an index into shown
top     = $c102                 ; the first one on screen
row     = $c103
cur     = $c104
nshown  = $c105                 ; how many are shown
qlen    = $c106                 ; the length of the filter text
order   = $c107                 ; 0 directory order, 1 A-Z
sorted_ok = $c108               ; non-zero once sorted holds the A-Z order
si      = $c109                 ; loop counters and scratch for refilter, sort_az and match
skey    = $c10b
mstart  = $c10c
fchar   = $c10d
dnum    = $c10e
dlead   = $c10f
query   = $c110                 ; QMAX bytes, the filter text, folded
slo     = $c120                 ; sort_az's search bounds
shi     = $c121
smid    = $c122
sorted  = $c200                 ; the visible entries' slot numbers, A-Z
shown   = $c300                 ; the ones listed: visible or sorted, filtered
name_lo = $c400                 ; each directory slot's address, by slot number
name_hi = $c500

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
        sta qlen
        sta order
        sta sorted_ok
        jsr scan
        jmp refresh

loop:
        jsr GETIN
        bne key
menu_idle:
        jmp loop

key:
        cmp #KEY_DOWN
        beq down
        cmp #KEY_UP
        bne +
        jmp up
+       cmp #KEY_RETURN
        bne +
        jmp start
+       cmp #KEY_STOP
        bne +
        jmp leave
+       cmp #KEY_F1
        beq toggle
        cmp #KEY_DEL
        beq erase
        cmp #KEY_HOME
        beq clear
        cmp #KEY_CLR
        beq clear
        jsr typable
        bcc loop
        ldx qlen
        cpx #QMAX
        bcs loop                ; the text is full
        sta query,x
        inc qlen
        jmp refresh

toggle:
        lda order
        eor #1
        sta order
        beq refresh
        lda sorted_ok
        bne refresh
        jsr sort_az             ; once: the directory never changes while the menu runs
        jmp refresh

erase:
        lda qlen
        beq loop
        dec qlen
        jmp refresh

clear:
        lda #0
        sta qlen

; The text or the order changed: list again from the first match.
refresh:
        jsr refilter
        lda #0
        sta sel
        sta top
        jmp redraw

down:
        ldx sel
        inx
        cpx nshown
        bcc +
        jmp loop                ; already on the last entry
+       stx sel
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
        bne +
        jmp loop                ; already on the first entry
+       dec sel
        lda sel
        cmp top
        bcs redraw
        sta top
        jmp redraw

; --- starting the selected entry --------------------------------------------------------------

start:
        lda nshown
        bne +
        jmp loop                ; nothing listed, nothing to start
+       ldx sel
        lda shown,x
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
        lda ptr
        sta name_lo,x
        lda ptr+1
        sta name_hi,x
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

; ptr = the directory slot numbered A, from the addresses scan recorded. Clobbers X.
slot_ptr:
        tax
        lda name_lo,x
        sta ptr
        lda name_hi,x
        sta ptr+1
        rts

; Fold a name or key byte so letters compare equal whatever their case: $61-$7A (what an
; uppercase file stem becomes) and $C1-$DA (shifted letters) to $41-$5A. Keeps X and Y.
fold:
        cmp #$c1
        bcc _low
        cmp #$db
        bcs _done
        and #$7f
        rts
_low:
        cmp #$61
        bcc _done
        cmp #$7b
        bcs _done
        and #$df
_done:
        rts

; Whether key A goes into the filter text. C set, with A folded, if it does: the printable
; characters and the shifted letters, but not the quote, which would put CHROUT in quote mode.
typable:
        cmp #$c1
        bcc _low
        cmp #$db
        bcs _no
        and #$7f
        sec
        rts
_low:
        cmp #$20
        bcc _no
        cmp #$60
        bcs _no
        cmp #$22
        beq _no
        sec
        rts
_no:
        clc
        rts

; shown = the visible entries in the order in use, keeping those whose name contains the text.
refilter:
        lda #0
        sta nshown
        sta si
_next:
        lda si
        cmp count
        bcs _done
        tax
        lda order
        beq _dir
        lda sorted,x
        jmp _have
_dir:
        lda visible,x
_have:
        sta skey
        jsr match
        bcc _skip
        ldy nshown
        lda skey
        sta shown,y
        inc nshown
_skip:
        inc si
        jmp _next
_done:
        rts

; C set if the name in slot A contains the text, comparing folded. The text is folded already.
match:
        jsr slot_ptr
        lda qlen
        beq _yes
        lda #0
        sta mstart
_try:
        lda mstart
        clc
        adc qlen
        cmp #QMAX + 1
        bcs _no                 ; the text no longer fits in what is left of the name
        ldx #0
        ldy mstart
_cmp:
        lda (ptr),y
        beq _no                 ; the name ends first, so no later start fits either
        jsr fold
        cmp query,x
        bne _again
        iny
        inx
        cpx qlen
        bne _cmp
_yes:
        sec
        rts
_again:
        inc mstart
        jmp _try
_no:
        clc
        rts

; sorted = visible ordered A-Z by folded name. A binary insertion sort: each entry goes after every
; entry it doesn't sort before, so names equal once folded keep directory order, and a cart of 200
; entries costs about 1600 comparisons rather than the 20000 a plain insertion sort can take.
sort_az:
        lda #0
        sta si
_next:
        lda si
        cmp count
        bcs _done
        tax
        lda visible,x
        sta skey
        lda #0
        sta slo
        stx shi                 ; search sorted[0..si) for the first entry skey sorts before
_search:
        lda slo
        cmp shi
        bcs _found
        clc
        adc shi
        ror a                   ; (lo + hi) / 2, the carry from the add coming in at bit 7
        sta smid
        tay
        lda sorted,y
        jsr before
        bcc _right
        lda smid
        sta shi
        jmp _search
_right:
        ldy smid
        iny
        sty slo
        jmp _search
_found:
        ldy si
_shift:
        cpy slo
        beq _put
        lda sorted-1,y
        sta sorted,y
        dey
        jmp _shift
_put:
        lda skey
        sta sorted,y
        inc si
        jmp _next
_done:
        lda #1
        sta sorted_ok
        rts

; C set if the name in slot skey sorts strictly before the name in slot A, comparing folded.
before:
        tax
        lda name_lo,x
        sta dst
        lda name_hi,x
        sta dst+1
        lda skey
        jsr slot_ptr
        ldy #0
_cmp:
        lda (dst),y
        jsr fold
        sta fchar
        lda (ptr),y
        jsr fold
        cmp fchar
        bcc _yes
        bne _no
        ora #0
        beq _no                 ; both names end here: equal
        iny
        cpy #16
        bne _cmp
_no:
        clc
        rts
_yes:
        sec
        rts

; Print the string at A (low), Y (high), up to its NUL.
say:
        sta dst
        sty dst+1
        ldy #0
-       lda (dst),y
        beq +
        jsr CHROUT
        iny
        bne -
+       rts

; Print A in decimal, without leading zeros.
decimal:
        sta dnum
        lda #0
        sta dlead
        ldx #0
_digit:
        ldy #"0"
_sub:
        lda dnum
        cmp tens,x
        bcc _emit
        sbc tens,x
        sta dnum
        iny
        bne _sub
_emit:
        cpy #"0"
        bne _put
        lda dlead
        bne _put
        cpx #2
        bne _skip               ; a leading zero, and not the last digit
_put:
        tya
        jsr CHROUT
        lda #1
        sta dlead
_skip:
        inx
        cpx #3
        bne _digit
        rts

tens:
        .byte 100, 10, 1

; Row 0 the cart name, row 1 the filter text, rows 2-21 the list, row 23 the count and order.
; Nothing prints a RETURN on the last two rows, which would scroll the screen.
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
        lda qlen
        beq _bare
        lda #<find_text
        ldy #>find_text
        jsr say
        ldx #0
-       lda query,x
        jsr putc
        inx
        cpx qlen
        bne -
_bare:
        lda #KEY_RETURN
        jsr CHROUT
        lda count
        bne _some
        lda #<no_entries
        ldy #>no_entries
        jmp say
_some:
        lda nshown
        bne _rows
        lda #<no_matches
        ldy #>no_matches
        jsr say
        jmp _status
_rows:
        lda #0
        sta row
_row:
        lda row
        clc
        adc top
        cmp nshown
        bcs _status             ; past the last entry
        sta cur
        cmp sel
        bne +
        lda #$12                ; reverse on: the selected entry
        jsr CHROUT
+       ldx cur
        lda shown,x
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
_status:
        clc
        ldx #STATUS
        ldy #0
        jsr PLOT
        lda nshown
        jsr decimal
        lda #<of_text
        ldy #>of_text
        jsr say
        lda count
        jsr decimal
        lda #<cart_order
        ldy #>cart_order
        ldx order
        beq +
        lda #<az_order
        ldy #>az_order
+       jmp say

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
no_matches:
        .null "no matches"
find_text:
        .null "find: "
of_text:
        .null " of "
cart_order:
        .null "   cart order"
az_order:
        .null "   a-z order"

        .include "launch.s"

        .cerror * > $a000, "the menu outgrows the 8 KiB loader slot"
        .fill $a000 - *, $ff
