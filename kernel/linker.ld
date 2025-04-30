KERNEL_OFFSET = 0xffff800000000000;

SECTIONS {
    . = 0x10000;
    . += KERNEL_OFFSET;

    _kernel_start = .;

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_OFFSET) {
        _text_start = .;
        *(.text .text.* .ltext .ltext.*)
        . = ALIGN(4K);
        _text_end = .;
    }



    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        _rodata_start = .;
        *(.rodata .rodata.* .lrodata .lrodata.*)
        . = ALIGN(4K);

        TEXT_START = .;
        QUAD(_text_start);
        TEXT_SIZE = .;
        QUAD(_text_end - _text_start);

        DATA_START = .;
        QUAD(_data_start);
        DATA_SIZE = .;
        QUAD(_data_end - _data_start);

        RODATA_START = .;
        QUAD(_rodata_start);
        RODATA_SIZE = .;
        QUAD(_rodata_end - _rodata_start);

        BSS_START = .;
        QUAD(_bss_start);
        BSS_SIZE = .;
        QUAD(_bss_end - _bss_start);
        _rodata_end = .;
    }

    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_OFFSET) {
        _data_start = .;
        *(.data .data.* .ldata .ldata.*)
        . = ALIGN(4K);
        _data_end = .;
    }

    .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_OFFSET) {
        _bss_start = .;
        *(.bss .bss.* .lbss .lbss.*)
        *(COMMON)
        . = ALIGN(4K);
        _bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.dynamic)
    }

    _kernel_end = .;
}
