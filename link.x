SECTIONS {
    . = 0x0;

    /*
     * The PrgCode output section name comes from the CMSIS-Pack flash algo
     * templates and armlink. It is used here because several tools that work
     * with these flash algos expect this section name.
     *
     * All input sections are combined into PrgCode because RWPI using R9 is not
     * currently stable in Rust, thus having separate PrgData sections that the
     * debug host might locate at a different offset from PrgCode is not safe.
     */
    PrgCode : {
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

    /*
     * Adding PrgData section in order to satisfy tools that need it.
     */
    PrgData : {
        KEEP(*(.PrgData))
        KEEP(*(.PrgData.*))

        . = ALIGN(4);
    }

    /* Description of the flash algorithm */
    DeviceData . : {
        /* The device data content is only for external tools,
         * and usually not referenced by the code.
         *
         * The KEEP statement ensures it's not removed by accident.
         */
        KEEP(*(DeviceData))
    }

    /DISCARD/ : {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}
