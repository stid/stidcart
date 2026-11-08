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
;
; The screen is layout C (D-041): a block logo, the list with a type icon per entry, a scroll bar and
; a pane describing the selection, drawn from a RAM copy of the ROM's mixed-case set that carries
; the menu's own glyphs. It wears the raster brand (D-048): the cartridge mark beside the logo and
; a strip of five bands under the cart name, both drawn by a raster interrupt.

GETIN   = $ffe4

SCREEN  = $0400
COLOURS = $d800                 ; colour RAM
CHARSET = $d018                 ; VIC memory setup: $15 the upper-case set, $1E ours at $3800
SHIFT_LOCK = $0291              ; bit 7 set: SHIFT+C= doesn't switch the set
RASTER  = $d012
BORDER  = $d020
BACKGROUND = $d021
CHAR_ROM = $d800                ; the ROM's mixed-case set, with the character ROM banked in
CHAR_RAM = $3800                ; our copy, which the VIC reads in its bank 0
BLACK      = 0                  ; the colours the menu uses (D-040, D-041)
WHITE      = 1
PURPLE     = 4
YELLOW     = 7
ORANGE     = 8
LIGHT_RED  = 10
DARK_GREY  = 11
GREY       = 12
LIGHT_GREEN = 13
LIGHT_BLUE = 14
LIGHT_GREY = 15

; The menu's own screen codes, in $60-$7F, which names never produce (screen_code turns PETSCII
; $A0-$BF into "?"). Each badge code+$80 holds its inverse, for the selection bar.
ICON    = $60                   ; ten: a badge per kind, left then right half (program, 8K, 16K,
                                ; Ultimax, multi-bank)
RULE    = $6a
TRACK   = $6b
THUMB   = $6c
QUARTER = $6d                   ; sixteen: the quarter-block cells of the logo
MARK    = $ea                   ; nine: the cartridge mark, 3x3 multicolour cells row by row. These are
                                ; reversed codes of the glyphs above, which names never reach either

BANK    = $de00                 ; EasyFlash bank register
CONTROL = $de02                 ; EasyFlash mode register
MODE_OFF = $04
MODE_ULT = $05
MODE_8K  = $06
MODE_16K = $07

DIR     = $a000                 ; EasyFS2 directory, 24-byte slots
EFNAME  = $bb08                 ; cart name, 16 bytes PETSCII, NUL padded
SLOT    = 24
ROWS    = 16                    ; entries shown at once, on screen rows 5-20
LIST    = 5                     ; the first list row
TRACK_COL = 21                  ; the scroll bar's column
PANE    = 23                    ; the details pane's first column
VALUE   = 29                    ; where its values start
STATUS  = 22                    ; the screen row with the position and the order
FIND    = 23                    ; the screen row with the filter text and its matches
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
srow    = $c150                 ; the screen row a list row goes to
kind    = $c151                 ; the entry's kind: 0 program, 1 8K, 2 16K, 3 Ultimax, 4 multi-bank
ipen    = $c152                 ; the icon's colour
old     = $c153                 ; the selection before a move
nlo     = $c154                 ; 16-bit number for mul8, div16 and dec16
nhi     = $c155
den     = $c156
ma      = $c157
mlo     = $c158
mhi     = $c159
tmpd    = $c15a
tlen    = $c15b                 ; the scroll bar's thumb: its length and its first row
tat     = $c15c
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
        jsr charset_init        ; interrupts are still off, as it needs
        jsr irq_on
        cli
        lda #BLACK
        sta BORDER
        sta BACKGROUND
        lda #$1e                ; the screen at $0400, our set at $3800 (D-041)
        sta CHARSET
        lda #$80
        sta SHIFT_LOCK          ; SHIFT+C= would switch the VIC to $3000
        lda #0
        sta qlen
        sta order
        sta sorted_ok
        jsr scan
        jsr draw_frame
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
        jmp stop
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
        jsr vsync
        jsr draw_list
        jsr draw_track
        jsr draw_pane
        jsr draw_status
        jsr draw_find
        jmp loop

down:
        ldx sel
        inx
        cpx nshown
        bcc +
        jmp loop                ; already on the last entry
+       lda sel
        sta old
        stx sel
        txa
        sec
        sbc top
        cmp #ROWS
        bcc moved
        inc top
        jmp scrolled

up:
        lda sel
        bne +
        jmp loop                ; already on the first entry
+       sta old
        dec sel
        lda sel
        cmp top
        bcs moved
        sta top

; The view moved: every list row changes, and the thumb.
scrolled:
        jsr vsync
        jsr draw_list
        jsr draw_track
        jmp described

; The selection moved within the view: only its old and new rows change (AC-111).
moved:
        jsr vsync
        lda old
        jsr draw_index
        lda sel
        jsr draw_index
described:
        jsr draw_pane
        jsr draw_status
        jmp loop

; --- starting the selected entry --------------------------------------------------------------

; RUN/STOP: the interrupts as the KERNAL left them, then BASIC.
stop:
        jsr irq_off
        jmp leave

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
        jsr irq_off             ; before any bank is mapped in, and as the stock menu left it
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

start_prg:
        jsr irq_off             ; read_load maps the program's bank in
        jsr prg_params
        jsr read_load
        jmp copy_prg

; The copy parameters of the program in the slot at ptr, its type in cur. Bytes 17-18 are its
; first bank and 19-20 its offset in the bank, bit 13 selecting ROMH, so in 16K mode its first
; byte is at $8000 + (offset & $3fff). Types $01-$03 differ only in where a bank runs out.
prg_params:
        ldy #17
        lda (ptr),y
        sta p_bank
        iny
        lda (ptr),y
        sta p_bank+1
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
        rts

copy_prg:
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

; --- the raster interrupt (D-048) ---------------------------------------------------------------
;
; One interrupt a frame draws the brand. From IRQ_LINE it turns multicolour on for the logo rows,
; where only the mark's cells have colour 8-15, and rewrites $D022 every three lines so the mark's
; label shows the five bands. It turns multicolour off at the end of row 2, then sets the background
; for one line each on 84-88: the strip under the cart name. Each colour is loaded before its wait,
; so the store lands in the line's first cycles, before its visible pixels. Then it runs the
; KERNAL's own work at $EA31, the clock and the keyboard, which the CIA timer no longer triggers:
; nothing else can hold the bars up.
;
; The handler runs from this ROM. Whatever maps another bank in masks interrupts first, and every
; way out calls irq_off before it does.

IRQ_LINE = 46                   ; a few lines before the label's first band
MARK_TOP = 54                   ; the label's first line
MC_OFF  = 74                    ; the last line of row 2, after which no cell may be multicolour
STRIP   = 84                    ; row 4 starts on the badline at 83: one band a line from 84

irq_on:
        lda #$7f                ; the CIA timer's interrupt off: this one runs the KERNAL's work
        sta $dc0d
        lda $dc0d
        lda #<irq
        sta $0314
        lda #>irq
        sta $0315
        lda #IRQ_LINE
        sta RASTER
        lda $d011
        and #$7f                ; bit 8 of the line
        sta $d011
        lda #$ff
        sta $d019
        lda #1
        sta $d01a
        rts

; Put the interrupts back as the KERNAL set them, and leave them masked: the caller is on its way out
; and maps other banks in next.
irq_off:
        sei
        lda #0
        sta $d01a
        lda #$ff
        sta $d019
        lda #<$ea31
        sta $0314
        lda #>$ea31
        sta $0315
        lda #$81
        sta $dc0d
        rts

irq:
        lda #$ff
        sta $d019
        bit $d011
        bmi _late               ; lines 256-311
        lda RASTER
        cmp #MARK_TOP
        bcs _late               ; held up past the first band: no bars this frame
        lda $d016
        ora #$10
        sta $d016
        ldx #0
_label:
        ldy mark_lines,x
        lda band_colours,x
-       cpy RASTER
        bne -
        sta $d022
        inx
        cpx #5
        bne _label
        ldy #MC_OFF
-       cpy RASTER
        bne -
        ldx #5                  ; past the mark's columns on that line
-       dex
        bne -
        lda $d016
        and #$ef
        sta $d016
        ldy #STRIP
        ldx #0
_strip:
        lda band_colours,x
-       cpy RASTER
        bne -
        sta BACKGROUND
        iny
        inx
        cpx #6
        bne _strip
        jmp $ea31

; Entered too late to draw the bars: make sure nothing of them is left on, and go on.
_late:
        lda $d016
        and #$ef
        sta $d016
        lda #BLACK
        sta BACKGROUND
        jmp $ea31

; --- the screen -------------------------------------------------------------------------------
;
; Layout C (D-041): rows 0-2 the logo, row 3 the cart name, rows 4 and 21 rules, rows 5-20 the list
; in columns 1-20 with the scroll bar in column 21 and the details pane in columns 23-38, row 22 the
; position and order, row 23 the filter text, row 24 the help bar. Everything is written straight
; into screen and colour RAM, each cell once per update, never cleared first: a cleared cell can
; be on screen for a frame, which is the flicker D-041 removes.

; Copy the ROM's mixed-case set to $3800 and put the menu's glyphs into it. Runs with interrupts
; off: the character ROM replaces I/O while it is banked in, so nothing here may touch $Dxxx.
charset_init:
        lda $01
        pha
        lda #$33                ; the character ROM at $D000, BASIC, KERNAL and the cart as they were
        sta $01
        lda #<CHAR_ROM
        sta ptr
        lda #>CHAR_ROM
        sta ptr+1
        lda #<CHAR_RAM
        sta dst
        lda #>CHAR_RAM
        sta dst+1
        ldx #8                  ; 2 KiB
        ldy #0
-       lda (ptr),y
        sta (dst),y
        iny
        bne -
        inc ptr+1
        inc dst+1
        dex
        bne -
        pla
        sta $01
        ldx #0
-       lda glyphs,x
        sta CHAR_RAM + ICON * 8,x
        inx
        cpx #glyphs_end - glyphs
        bne -
        ldx #0
-       lda glyphs,x            ; the badges' inverses, for the selection bar
        eor #$ff
        sta CHAR_RAM + (ICON + $80) * 8,x
        inx
        cpx #10 * 8
        bne -
        ldx #0
-       lda mark_glyphs,x
        sta CHAR_RAM + MARK * 8,x
        inx
        cpx #mark_glyphs_end - mark_glyphs
        bne -
        lda #DARK_GREY
        sta $d023               ; multicolour %10: the mark's shell
        rts

; Wait until the beam is below the visible screen, so an update starts where no row is being shown.
vsync:
-       bit $d011
        bmi -                   ; lines 256-311: wait for the next frame
-       lda RASTER
        cmp #251
        bcc -
        rts

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

; Write spaces from scr+Y up to scr+A-1, in colour pen, reversed if rvs is $80.
pad:
        sta wmax
-       cpy wmax
        bcs +
        lda #" " & $3f
        ora rvs
        sta (scr),y
        lda pen
        sta (clr),y
        iny
        bne -
+       rts

; Write the PETSCII text at dst from scr, in colour pen, reversed if rvs is $80: up to its NUL, or
; up to X characters with write_n. Y ends as the number written.
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
; mixed-case set, $41-$5A show lower case and $61-$7A and $C1-$DA upper case. $A0-$BF, the PETSCII
; graphics whose codes are the menu's own glyphs, show as "?" too (D-041). Keeps X and Y.
screen_code:
        cmp #$20
        bcc _bad
        cmp #$40
        bcc _done               ; $20-$3F: the same
        cmp #$60
        bcc _less40             ; $40-$5F: $00-$1F
        cmp #$80
        bcc _less20             ; $60-$7F: $40-$5F
        cmp #$c0
        bcc _bad                ; $80-$9F control codes, $A0-$BF the menu's glyphs
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

; --- building a line of text in sbuf ----------------------------------------------------------

; Append A to the text. Keeps X and Y.
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
        sta nlo
        lda #0
        sta nhi
; Append nlo/nhi in decimal, without leading zeros.
dec16:
        lda #0
        sta dlead
        ldy #0
_pow:
        ldx #"0"
_sub:
        lda nlo
        sec
        sbc pow_lo,y
        sta tmpd
        lda nhi
        sbc pow_hi,y
        bcc _emit
        sta nhi
        lda tmpd
        sta nlo
        inx
        bne _sub
_emit:
        cpx #"0"
        bne _put
        lda dlead
        bne _put
        cpy #4
        bne _skip               ; a leading zero, and not the last digit
_put:
        txa
        jsr emit
        lda #1
        sta dlead
_skip:
        iny
        cpy #5
        bne _pow
        rts

; Append A as two hex digits, in upper case.
hex2:
        pha
        lsr a
        lsr a
        lsr a
        lsr a
        tax
        lda hex_digits,x
        jsr emit
        pla
        and #$0f
        tax
        lda hex_digits,x
        jmp emit

; Start the text afresh.
begin:
        lda #0
        sta sx
        rts

; End the text and write it at column A of row X, in colour pen, then pad to column 39.
finish:
        pha
        lda #0
        jsr emit
        pla
        sta tmpd
        jsr at
        lda #<sbuf
        ldy #>sbuf
        jsr say
        lda #39
        sec
        sbc tmpd
        jmp pad

; --- arithmetic for the scroll bar -------------------------------------------------------------

; nlo/nhi = A * den.
mul8:
        sta ma
        lda #0
        sta nlo
        sta nhi
        sta mhi
        lda den
        sta mlo
-       lsr ma
        bcc +
        clc
        lda nlo
        adc mlo
        sta nlo
        lda nhi
        adc mhi
        sta nhi
+       asl mlo
        rol mhi
        lda ma
        bne -
        rts

; nlo/nhi = nlo/nhi / den, rounded down.
div16:
        lda #0
        ldx #16
-       asl nlo
        rol nhi
        rol a
        bcs +
        cmp den
        bcc ++
+       sbc den
        inc nlo
+       dex
        bne -
        rts

; --- the frame, drawn once -------------------------------------------------------------------

draw_frame:
        lda #0
        sta rvs
        ; rows 0-2: the logo
        ldx #0
_logo:
        lda logo_colours,x
        sta pen
        stx row
        lda #7
        jsr at
        ldx row
        lda logo_lo,x
        sta dst
        lda logo_hi,x
        sta dst+1
        ldy #0
-       lda (dst),y
        sta (scr),y
        lda pen
        sta (clr),y
        iny
        cpy #31
        bne -
        ldx row
        inx
        cpx #3
        bne _logo

        ; rows 0-2, columns 2-4: the mark, in multicolour cells (8 + their own colour, the pins')
        ldx #0
_mark:
        stx row
        lda #2
        jsr at
        lda row
        asl a
        adc row                 ; row * 3, carry clear
        adc #MARK
        ldy #0
-       sta (scr),y
        tax
        lda #8 + YELLOW
        sta (clr),y
        txa
        clc
        adc #1
        iny
        cpy #3
        bne -
        ldx row
        inx
        cpx #3
        bne _mark

        ; row 3: the cart name, centred
        lda #LIGHT_GREY
        sta pen
        ldy #0
-       lda EFNAME,y
        beq +
        iny
        cpy #16
        bne -
+       sty nlen
        lda #40
        sec
        sbc nlen
        lsr a
        ldx #3
        jsr at
        lda #<EFNAME
        sta dst
        lda #>EFNAME
        sta dst+1
        ldx #16
        jsr write_n

        ; row 4: spaces, where the raster strip shows; row 21: the rule
        ldx #4
        lda #$20
        jsr line
        lda #DARK_GREY
        sta pen
        ldx #21
        lda #RULE
        jsr line

        ; row 24: the help bar
        lda #GREY
        sta pen
        ldx #HELP
        lda #$a0                ; a reverse space
        jsr line
        lda #$80
        sta rvs
        ldx #HELP
        lda #1
        jsr at
        lda #<help_text
        ldy #>help_text
        jsr say
        lda #0
        sta rvs
        rts

; --- the list ---------------------------------------------------------------------------------

; Every list row.
draw_list:
        lda #0
        sta row
-       jsr list_row
        inc row
        lda row
        cmp #ROWS
        bne -
        rts

; The list row of entry A in shown, which is on screen.
draw_index:
        sec
        sbc top
        sta row

; List row `row` (0-15): columns 1-20 of screen row 5 + row.
list_row:
        lda row
        clc
        adc #LIST
        sta srow
        lda #0
        sta rvs
        lda #LIGHT_GREY
        sta pen
        lda count
        bne +
        lda #<no_entries
        ldy #>no_entries
        jmp _message
+       lda nshown
        bne _entry
        lda #<no_matches
        ldy #>no_matches
_message:
        ldx row
        bne _blank              ; the message goes on the first row only
        pha
        tya
        pha
        ldx srow
        lda #1
        jsr at
        ldy #0
        lda #3
        jsr pad
        ldx srow
        lda #4
        jsr at
        pla
        tay
        pla
        jsr say
        lda #17
        jmp pad
_entry:
        lda row
        clc
        adc top
        cmp nshown
        bcc _listed
_blank:
        ldx srow
        lda #1
        jsr at
        ldy #0
        lda #20
        jmp pad
_listed:
        sta cur
        tax
        lda shown,x
        jsr slot_ptr
        jsr entry_kind
        ldx kind
        lda kind_colours,x
        sta ipen
        lda cur
        cmp sel
        bne +
        lda #$80                ; the selection: a cyan reverse bar over columns 1-20
        sta rvs
        lda #ORANGE             ; the selection bar: the brand's accent (D-048)
        sta pen
        sta ipen
+       ldx srow
        lda #1
        jsr at
        lda kind                ; columns 1-2, the badge
        asl a
        clc
        adc #ICON
        ora rvs
        ldy #0
        sta (scr),y
        ora #1
        iny
        sta (scr),y
        lda ipen
        sta (clr),y
        dey
        sta (clr),y
        ldy #2
        lda #3
        jsr pad                 ; column 3
        ldx srow
        lda #4
        jsr at
        lda ptr
        sta dst
        lda ptr+1
        sta dst+1
        ldx #16
        jsr write_n             ; columns 4-19, the name
        lda #17
        jmp pad                 ; to column 20

; kind = the kind of the entry in the slot at ptr: 0 program ($01-$03), 1 8K ($10), 2 16K ($11),
; 3 Ultimax ($12, $13), 4 multi-bank ($1C-$1E).
entry_kind:
        ldy #16
        lda (ptr),y
        and #$1f
        ldx #0
        cmp #$04
        bcc +
        inx
        cmp #$10
        beq +
        inx
        cmp #$11
        beq +
        inx
        cmp #$12
        beq +
        cmp #$13
        beq +
        inx
+       stx kind
        rts

; --- the scroll bar ---------------------------------------------------------------------------

; Column 21 of the list rows: the track, with a thumb of round(256/n) cells (at least 1)
; round((16-len)*top/(n-16)) rows down when more than 16 are listed (AC-109).
draw_track:
        lda #0
        sta tlen
        sta tat
        lda nshown
        cmp #ROWS + 1
        bcc _draw
        sta den                 ; len = (256 + n/2) / n
        lsr a
        sta nlo
        lda #1
        sta nhi
        jsr div16
        lda nlo
        bne +
        lda #1
+       sta tlen
        lda nshown              ; at = ((16-len)*top + span/2) / span, span = n-16
        sec
        sbc #ROWS
        sta den
        lda top
        sta tmpd
        lda #ROWS
        sec
        sbc tlen
        ldx den
        stx etmp
        ldx tmpd
        stx den
        jsr mul8                ; (16-len) * top
        lda etmp
        sta den
        lsr a
        clc
        adc nlo
        sta nlo
        bcc +
        inc nhi
+       jsr div16
        lda nlo
        sta tat
_draw:
        lda #0
        sta row
_cell:
        lda row
        clc
        adc #LIST
        tax
        lda #TRACK_COL
        jsr at
        ldx #TRACK
        lda #DARK_GREY
        sta pen
        lda row
        sec
        sbc tat
        bcc +                   ; above the thumb
        cmp tlen
        bcs +                   ; below it
        ldx #THUMB
        lda #LIGHT_GREY
        sta pen
+       txa
        ldy #0
        sta (scr),y
        lda pen
        sta (clr),y
        inc row
        lda row
        cmp #ROWS
        bne _cell
        rts

; --- the details pane --------------------------------------------------------------------------

; Rows 6-14 of columns 23-38: the selection's name, then type, size, load address or bank, and
; how it starts, on every other row (AC-108).
draw_pane:
        lda #0
        sta rvs
        lda nshown
        bne _some
        ldx #6
-       stx row
        lda #PANE
        jsr at
        ldy #0
        lda #16
        jsr pad
        ldx row
        inx
        inx
        cpx #16
        bne -
        rts
_some:
        ldx sel
        lda shown,x
        jsr slot_ptr
        jsr entry_kind
        lda #WHITE
        sta pen
        ldx #6
        lda #PANE
        jsr at
        lda ptr
        sta dst
        lda ptr+1
        sta dst+1
        ldx #16
        jsr write_n
        lda #16
        jsr pad

        ; type
        ldx #8
        lda #<type_text
        ldy #>type_text
        jsr fact
        ldx kind
        lda kind_lo,x
        ldy kind_hi,x
        jsr append
        ldx #8
        jsr value

        ; size in KB, rounded up: (bytes + 1023) / 1024
        ldx #10
        lda #<size_text
        ldy #>size_text
        jsr fact
        ldy #21
        lda (ptr),y
        clc
        adc #$ff
        iny
        lda (ptr),y
        adc #$03
        sta nlo
        iny
        lda (ptr),y
        adc #0
        sta nhi
        lsr nhi
        ror nlo
        lsr nhi
        ror nlo
        jsr dec16
        lda #"K"
        jsr emit
        ldx #10
        jsr value

        lda kind
        beq _program

        ; a cartridge: its first bank, and a reset
        ldx #12
        lda #<bank_text
        ldy #>bank_text
        jsr fact
        ldy #17
        lda (ptr),y
        sta nlo
        iny
        lda (ptr),y
        sta nhi
        jsr dec16
        ldx #12
        jsr value
        ldx #14
        lda #<start_text
        ldy #>start_text
        jsr fact
        lda #<reset_text
        ldy #>reset_text
        jsr append
        ldx #14
        jmp value

        ; a program: its load address, and RUN or SYS as start uses them
_program:
        ldy #16
        lda (ptr),y
        and #$1f
        sta cur
        jsr prg_params
        sei                     ; the raster handler runs from bank 0, which read_load maps out
        jsr read_load           ; moves ptr: everything from the slot is read by now
        cli
        ldx #12
        lda #<load_text
        ldy #>load_text
        jsr fact
        lda #"$"
        jsr emit
        lda p_load+1
        jsr hex2
        lda p_load
        jsr hex2
        ldx #12
        jsr value
        ldx #14
        lda #<start_text
        ldy #>start_text
        jsr fact
        lda p_load+1
        cmp #$08
        bcc _run
        bne _sys
        lda p_load
        cmp #$02
        bcc _run
_sys:
        lda #<sys_word
        ldy #>sys_word
        jsr append
        lda p_load
        sta nlo
        lda p_load+1
        sta nhi
        jsr dec16
        ldx #14
        jmp value
_run:
        lda #<run_word
        ldy #>run_word
        jsr append
        ldx #14
        jmp value

; Row X of the pane: the label at A (low), Y (high) in grey across columns 23-28, then start the
; value's text.
fact:
        sta dst
        sty dst+1
        lda dst
        pha
        lda dst+1
        pha
        lda #GREY
        sta pen
        lda #PANE
        jsr at
        pla
        sta dst+1
        pla
        sta dst
        jsr write
        lda #6
        jsr pad
        jmp begin

; Write the value's text in row X, columns 29-38, in the kind's colour.
value:
        lda #0
        jsr emit
        ldy kind
        lda kind_colours,y
        sta pen
        lda #VALUE
        jsr at
        lda #<sbuf
        ldy #>sbuf
        jsr say
        lda #10
        jmp pad

; --- the status and the filter text ------------------------------------------------------------

; Row 22: the selection's position among those listed, and the order in use (AC-110).
draw_status:
        lda #0
        sta rvs
        lda #GREY
        sta pen
        jsr begin
        lda nshown
        beq +
        lda sel
        clc
        adc #1
+       jsr decimal
        lda #<of_text
        ldy #>of_text
        jsr append
        lda nshown
        jsr decimal
        lda #<cart_order
        ldy #>cart_order
        ldx order
        beq +
        lda #<az_order
        ldy #>az_order
+       jsr append
        ldx #STATUS
        lda #1
        jmp finish

; Row 23: with text typed, a grey label, the text in yellow, and the matches out of the visible
; entries at the right; without, nothing.
draw_find:
        lda #0
        sta rvs
        lda #GREY
        sta pen
        ldx #FIND
        lda #1
        jsr at
        ldy #0
        lda qlen
        bne +
        lda #38
        jmp pad
+       lda #<find_text
        ldy #>find_text
        jsr say
        lda #YELLOW
        sta pen
        ldx #FIND
        lda #7
        jsr at
        lda #<query
        sta dst
        lda #>query
        sta dst+1
        ldx qlen
        jsr write_n
        tya
        pha
        jsr begin
        lda nshown
        jsr decimal
        lda #<of_text
        ldy #>of_text
        jsr append
        lda count
        jsr decimal
        lda #<match_text
        ldy #>match_text
        jsr append
        lda #0
        jsr emit
        lda #GREY               ; pad from the text to where the count starts, at 39 - its length
        sta pen
        lda #39 + 1
        sec
        sbc sx                  ; sx counts the NUL too
        sta nlen
        sec
        sbc #7
        tax
        pla
        tay
        txa
        jsr pad
        ldx #FIND
        lda nlen
        jsr at
        lda #<sbuf
        ldy #>sbuf
        jmp say

; --- data -------------------------------------------------------------------------------------

row_lo:
        .byte <(SCREEN + range(25) * 40)
row_hi:
        .byte >(SCREEN + range(25) * 40)

; The glyphs, in code order from ICON. The pixels are the approved mockup's badges (D-041).
glyphs:
        .byte %00000000, %01111111, %11000100, %11010101, %11000100, %11011101, %11011101, %01111111 ; PRG, left
        .byte %00000000, %11111110, %01000111, %01011111, %11010111, %01010111, %01000111, %11111110 ; PRG, right
        .byte %00000000, %01111111, %11110001, %11110101, %11110001, %11110101, %11110001, %01111111 ; 8K, left
        .byte %00000000, %11111110, %01011111, %01011111, %00111111, %01011111, %01011111, %11111110 ; 8K, right
        .byte %00000000, %01111111, %11101100, %11001101, %11101100, %11101101, %11000100, %01111111 ; 16K, left
        .byte %00000000, %11111110, %01010111, %11010111, %01001111, %01010111, %01010111, %11111110 ; 16K, right
        .byte %00000000, %01111111, %11010101, %11010101, %11010101, %11010101, %11000100, %01111111 ; ULT, left
        .byte %00000000, %11111110, %11000111, %11101111, %11101111, %11101111, %01101111, %11111110 ; ULT, right
        .byte %00000000, %01111111, %11110101, %11110101, %11111011, %11110101, %11110101, %01111111 ; XB, left
        .byte %00000000, %11111110, %00111111, %01011111, %00111111, %01011111, %00111111, %11111110 ; XB, right
        .byte %00000000, %00000000, %00000000, %11111111, %00000000, %00000000, %00000000, %00000000 ; rule
        .fill 8, %00010000      ; track
        .fill 8, %00111100      ; thumb
        ; The sixteen quarter-block cells: bit 3 top left, 2 top right, 1 bottom left, 0 bottom right.
        .for q := 0, q < 16, q += 1
        .fill 4, ((q >> 3) & 1) * $f0 | ((q >> 2) & 1) * $0f
        .fill 4, ((q >> 1) & 1) * $f0 | (q & 1) * $0f
        .next
glyphs_end:

; The cartridge mark (D-048) in multicolour pixels, two bits each: %01 the label ($D022, a band
; every three lines), %10 the shell ($D023, dark grey), %11 the pins (the cells' own colour, yellow).
; Nine cells, 3x3, row by row, for codes MARK to MARK+8.
mark_glyphs:
        .byte $00, $2a, $aa, $a5, $a5, $a5, $a5, $a5
        .byte $00, $aa, $aa, $55, $55, $55, $55, $55
        .byte $00, $a8, $aa, $5a, $5a, $5a, $5a, $5a
        .byte $a5, $a5, $a5, $a5, $a5, $a5, $a5, $a5
        .byte $55, $55, $55, $55, $55, $55, $55, $55
        .byte $5a, $5a, $5a, $5a, $5a, $5a, $5a, $5a
        .byte $a5, $a5, $aa, $2a, $02, $02, $02, $00
        .byte $55, $55, $aa, $aa, $aa, $eb, $eb, $00
        .byte $5a, $5a, $aa, $a8, $80, $80, $80, $00
mark_glyphs_end:

mark_lines:
        .byte 54, 57, 60, 63, 66        ; where each label band starts; none is a badline (51, 59, 67)
band_colours:
        .byte LIGHT_RED, ORANGE, YELLOW, LIGHT_GREEN, LIGHT_BLUE, BLACK  ; black ends the strip

; "STIDCART" in block letters on a 6x6 grid of quarter cells, three characters a letter and one
; between, as screen codes: the space, or QUARTER + the cell's four quarters (bit 3 top left).
logo_0:
        .byte QUARTER+7, QUARTER+12, QUARTER+12, $20, QUARTER+12, QUARTER+15, QUARTER+12, $20, QUARTER+4, QUARTER+15, QUARTER+8, $20, QUARTER+15, QUARTER+12, QUARTER+11, $20
        .byte QUARTER+7, QUARTER+12, QUARTER+12, $20, QUARTER+7, QUARTER+12, QUARTER+11, $20, QUARTER+15, QUARTER+12, QUARTER+11, $20, QUARTER+12, QUARTER+15, QUARTER+12
logo_1:
        .byte QUARTER+4, QUARTER+12, QUARTER+11, $20, $20, QUARTER+15, $20, $20, $20, QUARTER+15, $20, $20, QUARTER+15, $20, QUARTER+15, $20
        .byte QUARTER+15, $20, $20, $20, QUARTER+15, QUARTER+3, QUARTER+15, $20, QUARTER+15, QUARTER+3, QUARTER+14, $20, $20, QUARTER+15, $20
logo_2:
        .byte QUARTER+3, QUARTER+3, QUARTER+14, $20, $20, QUARTER+15, $20, $20, QUARTER+1, QUARTER+15, QUARTER+2, $20, QUARTER+15, QUARTER+3, QUARTER+14, $20
        .byte QUARTER+13, QUARTER+3, QUARTER+3, $20, QUARTER+15, $20, QUARTER+15, $20, QUARTER+15, QUARTER+4, QUARTER+11, $20, $20, QUARTER+15, $20
logo_lo:
        .byte <logo_0, <logo_1, <logo_2
logo_hi:
        .byte >logo_0, >logo_1, >logo_2
logo_colours:
        .byte WHITE, WHITE, WHITE       ; white beside the colourful mark (D-048); multicolour rows need 0-7

kind_colours:
        .byte LIGHT_GREEN, YELLOW, LIGHT_RED, PURPLE, LIGHT_BLUE
kind_lo:
        .byte <kind_0, <kind_1, <kind_2, <kind_3, <kind_4
kind_hi:
        .byte >kind_0, >kind_1, >kind_2, >kind_3, >kind_4
kind_0: .null "program"
kind_1: .null "8K cart"
kind_2: .null "16K cart"
kind_3: .null "Ultimax"
kind_4: .null "multi-bank"

hex_digits:
        .text "0123456789ABCDEF"
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
match_text:
        .null " match"
cart_order:
        .null "   cart order"
az_order:
        .null "   a-z order"
type_text:
        .null "type"
size_text:
        .null "size"
load_text:
        .null "load"
bank_text:
        .null "bank"
start_text:
        .null "start"
reset_text:
        .null "reset"
run_word:
        .null "RUN"
sys_word:
        .null "SYS "

        .include "launch.s"

        .cerror * > $a000, "the menu outgrows the 8 KiB loader slot"
        .fill $a000 - *, $ff
