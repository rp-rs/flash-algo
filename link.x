SECTIONS {
    . = 0x0;

    .text : {
        KEEP(*(.entry))
        KEEP(*(.entry.*))

        *(.text)
        *(.text.*)

        *(.rodata)
        *(.rodata.*)

        *(.data)
        *(.data.*)

        *(.sdata)
        *(.sdata.*)
        
        *(.bss)
        *(.bss.*)

        *(.uninit)
        *(.uninit.*)

        . = ALIGN(4);
    }

    /DISCARD/ : {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}
