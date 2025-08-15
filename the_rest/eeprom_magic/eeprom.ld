SECTIONS
{
  /* Ensure a dedicated eeprom section with a global start symbol. */
  .eeprom_data : ALIGN(4)
  {
    __eeprom_data_start = .;
    KEEP(*(.eeprom_data))
    __eeprom_data_end = .;
  }
}
