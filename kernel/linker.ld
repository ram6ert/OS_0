KERNEL_OFFSET = 0xffff800000000000;

SECTIONS {
    . = 0x10000;
    . += KERNEL_OFFSET;

    _kernel_start = .;
    _text_start = .;

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_OFFSET) {
        *(.text .text.* .ltext .ltext.*)
        . = ALIGN(4K);
    }

    _text_end = .;

    _rodata_start = .;

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        *(.rodata .rodata.* .lrodata .lrodata.*)
        . = ALIGN(4K);
    }

    _rodata_end = .;

    _data_start = .;

    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_OFFSET) {
        *(.data .data.* .ldata .ldata.*)
        . = ALIGN(4K);
    }

    _data_end = .;

    _bss_start = .;

    .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_OFFSET) {
        *(.bss .bss.* .lbss .lbss.*)
        *(COMMON)
        . = ALIGN(4K);
    }

    _bss_start = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.dynamic)
    }

    _kernel_end = .;
}
