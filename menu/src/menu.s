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

GETIN   = $ffe4

SCREEN  = $0400
COLOURS = $d800                 ; colour RAM
CHARSET = $d018                 ; VIC memory setup: $15 the upper-case set, $17 mixed case
SHIFT_LOCK = $0291              ; bit 7 set: SHIFT+C= doesn't switch the set
BORDER  = $d020
BACKGROUND = $d021
BLACK      = 0                  ; the colours the menu uses (D-040)
CYAN       = 3
YELLOW     = 7
GREY       = 12
LIGHT_GREY = 15

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
HELP    = 24                    ; the screen row with the keys
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
scr     = $22                   ; 2 bytes: a screen cell, while the menu draws (BASIC is not running)
clr     = $24                   ; 2 bytes: the same cell's colour

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
wmax    = $c123                 ; draw's scratch
pen     = $c124                 ; the colour draw writes in
rvs     = $c125                 ; $80 when draw writes in reverse, else 0
sx      = $c126                 ; the length of the status text so far
etmp    = $c127
nlen    = $c128
sbuf    = $c130                 ; the status text, "200 of 200   cart order" and its NUL
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
        lda #BLACK
        sta BORDER
        sta BACKGROUND
        lda #$17                ; mixed case, so names keep their capitals (D-040)
        sta CHARSET
        lda #$80
        sta SHIFT_LOCK
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
        sta SHIFT_LOCK          ; the cart's reset runs before CINT, which would clear it (D-040)
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
        lda #14                 ; the C64's own colours and character set: basic_go enters
        sta BORDER              ; the reset after CINT, which would otherwise set them (D-040)
        lda #6
        sta BACKGROUND
        lda #$15
        sta CHARSET
        lda #0
        sta SHIFT_LOCK
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
; characters and the shifted letters, but not the quote (D-039, from when the menu drew through
; CHROUT, whose quote mode would print control codes as glyphs).
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

; --- the screen -------------------------------------------------------------------------------
;
; Row 0 the title bar, row 1 the filter text, rows 2-21 the list, row 23 the count and order, row
; 24 the help bar (D-040). Everything is written straight into screen and colour RAM: a 40-column
; row printed through CHROUT links into the next line, and reaching the last cell scrolls.

; Point scr at column A of screen row X, and clr at the same cell of colour RAM.
at:
        clc
        adc row_lo,x
        sta scr
        sta clr
        lda row_hi,x
        adc #0
        sta scr+1
        adc #>(COLOURS - SCREEN) ; the carry is clear: the screen ends below $0800
        sta clr+1
        rts

; Fill X cells from scr with screen code A, in colour pen.
fill:
        stx wmax
        ldy #0
-       sta (scr),y
        pha
        lda pen
        sta (clr),y
        pla
        iny
        cpy wmax
        bne -
        rts

; Fill the whole of row X with screen code A, in colour pen.
line:
        pha
        lda #0
        jsr at
        pla
        ldx #40
        jmp fill

; Write the PETSCII text at dst from scr, in colour pen, reversed if rvs is $80: up to its NUL, or
; up to X characters with write_n.
write:
        ldx #40
write_n:
        stx wmax
        ldy #0
-       cpy wmax
        beq +
        lda (dst),y
        beq +
        jsr screen_code
        ora rvs
        sta (scr),y
        lda pen
        sta (clr),y
        iny
        bne -
+       rts

; dst = the text at A (low), Y (high), then write it.
say:
        sta dst
        sty dst+1
        jmp write

; The screen code for PETSCII A, as CHROUT would print it, or "?" for a control code. In the
; mixed-case set the menu uses, $41-$5A show lower case and $61-$7A and $C1-$DA upper case. Keeps X
; and Y.
screen_code:
        cmp #$20
        bcc _bad
        cmp #$40
        bcc _done               ; $20-$3F: the same
        cmp #$60
        bcc _less40             ; $40-$5F: $00-$1F
        cmp #$80
        bcc _less20             ; $60-$7F: $40-$5F
        cmp #$a0
        bcc _bad                ; $80-$9F: control codes
        cmp #$c0
        bcc _less40             ; $A0-$BF: $60-$7F
        cmp #$ff
        beq _pi
        and #$7f                ; $C0-$FE: $40-$7E
        rts
_less40:
        sec
        sbc #$40
        rts
_less20:
        sec
        sbc #$20
        rts
_pi:
        lda #$5e
        rts
_bad:
        lda #"?"
_done:
        rts

; Append A to the status text. Keeps X and Y.
emit:
        stx etmp
        ldx sx
        sta sbuf,x
        inc sx
        ldx etmp
        rts

; Append the text at A (low), Y (high), up to its NUL.
append:
        sta dst
        sty dst+1
        ldy #0
-       lda (dst),y
        beq +
        jsr emit
        iny
        bne -
+       rts

; Append A in decimal, without leading zeros.
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
        jsr emit
        lda #1
        sta dlead
_skip:
        inx
        cpx #3
        bne _digit
        rts

tens:
        .byte 100, 10, 1

draw:
        ; row 0: the title bar, "stidcart" on the left and the cart name on the right
        lda #CYAN
        sta pen
        ldx #0
        lda #$a0                ; a reverse space
        jsr line
        lda #$80
        sta rvs
        ldx #0
        lda #1
        jsr at
        lda #<title_text
        ldy #>title_text
        jsr say
        ldy #0
-       lda EFNAME,y
        beq +
        iny
        cpy #16
        bne -
+       sty nlen
        lda #39
        sec
        sbc nlen
        ldx #0
        jsr at
        lda #<EFNAME
        sta dst
        lda #>EFNAME
        sta dst+1
        ldx #16
        jsr write_n

        ; row 1: the filter text, after a label, when there is any
        lda #0
        sta rvs
        ldx #1
        lda #" "
        jsr line
        lda qlen
        beq _list
        lda #GREY
        sta pen
        ldx #1
        lda #1
        jsr at
        lda #<find_text
        ldy #>find_text
        jsr say
        lda #YELLOW
        sta pen
        ldx #1
        lda #7
        jsr at
        lda #<query
        sta dst
        lda #>query
        sta dst+1
        ldx qlen
        jsr write_n

        ; rows 2-21: the list, or why there is none
_list:
        lda #0
        sta row
_row:
        lda #LIGHT_GREY
        sta pen
        lda #0
        sta rvs
        lda row
        clc
        adc #2
        tax
        lda #" "
        jsr line
        lda count
        bne +
        lda #<no_entries
        ldy #>no_entries
        jmp _say
+       lda nshown
        bne _entry
        lda #<no_matches
        ldy #>no_matches
_say:
        ldx row
        bne _next               ; the message goes on the first row only
        pha
        tya
        pha
        ldx #2
        lda #2
        jsr at
        pla
        tay
        pla
        jsr say
        jmp _next
_entry:
        lda row
        clc
        adc top
        cmp nshown
        bcs _next               ; past the last entry
        sta cur
        cmp sel
        bne _name
        lda #CYAN               ; the selected entry: a reverse bar over columns 1-38
        sta pen
        lda row
        clc
        adc #2
        tax
        lda #1
        jsr at
        lda #$a0
        ldx #38
        jsr fill
        lda #$80
        sta rvs
_name:
        lda row
        clc
        adc #2
        tax
        lda #2
        jsr at
        ldx cur
        lda shown,x
        jsr slot_ptr
        lda ptr
        sta dst
        lda ptr+1
        sta dst+1
        ldx #16
        jsr write_n
_next:
        inc row
        lda row
        cmp #ROWS
        bcs +
        jmp _row
+
        ; row 22 empty, row 23 the count and the order
        lda #0
        sta rvs
        ldx #22
        lda #" "
        jsr line
        lda #GREY
        sta pen
        ldx #STATUS
        lda #" "
        jsr line
        lda #0
        sta sx
        lda nshown
        jsr decimal
        lda #<of_text
        ldy #>of_text
        jsr append
        lda count
        jsr decimal
        lda #<cart_order
        ldy #>cart_order
        ldx order
        beq +
        lda #<az_order
        ldy #>az_order
+       jsr append
        lda #0
        jsr emit
        ldx #STATUS
        lda #1
        jsr at
        lda #<sbuf
        ldy #>sbuf
        jsr say

        ; row 24: the help bar
        ldx #HELP
        lda #$a0
        jsr line
        lda #$80
        sta rvs
        ldx #HELP
        lda #1
        jsr at
        lda #<help_text
        ldy #>help_text
        jmp say

row_lo:
        .byte <(SCREEN + range(25) * 40)
row_hi:
        .byte >(SCREEN + range(25) * 40)

title_text:
        .null "stidcart"
help_text:
        .null "return start  f1 sort  run/stop basic"
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
