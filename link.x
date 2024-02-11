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

     /* Section for data, specified by flashloader standard. */
    PrgData : {
      /*
       * We're explicitly putting a single object here (PRGDATA_Start in main.c) as this is required by some tools.
       * It is not used by this algorithm
       */
        KEEP(*(PrgData))

        . = ALIGN(4);
    }

    /* Description of the flash algorithm */
    DevDscr . : {
        /* The device data content is only for external tools,
         * and usually not referenced by the code.
         * All rules have exceptions: device data is used by this flash algo.
         *
         * The KEEP statement ensures it's not removed by accident.
         */
        KEEP(*(DeviceData))

        . = ALIGN(4);
    }

    /DISCARD/ : {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}
