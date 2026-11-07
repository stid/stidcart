; stidcart menu: lists a cart's entries and starts them.
; GPL-2.0-or-later, see COPYING at the repository root.
;
; Loaded at bank 0 ROML ($8000). The boot code enters through the first word in 16K mode, bank 0,
; with the KERNAL not initialised (docs/format/easyflash-cart-format.md, "How the boot code hands
; over to the menu loader").

        * = $8000
        .word entry

entry:
        sei
menu_idle:
        jmp menu_idle

        .include "launch.s"

        .cerror * > $a000, "the menu outgrows the 8 KiB loader slot"
        .fill $a000 - *, $ff
