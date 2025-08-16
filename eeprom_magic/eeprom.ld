/* MEMORY */
/* { */
    /* EEPROM (rwx) : ORIGIN = 0, LENGTH = 1K */
    /* FLASH  (rx)  : ORIGIN = 0x08000000, LENGTH = 28K */
    /* RAM    (rwx) : ORIGIN = 0x20000000, LENGTH = 2K */
/* } */

SECTIONS
{
    .eeprom_data :
    {
        __eeprom_data_start = .;
        KEEP(*(.eeprom_data))
        __eeprom_data_end = .;
    }
    /* } > EEPROM */
}
