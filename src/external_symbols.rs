// Symbols defined in the linker script
extern "C" {
    pub fn kernel_start();
    pub fn kernel_end();
    pub fn section_text_start();
    pub fn section_rodata_start();
    pub fn section_data_start();
    pub fn section_bss_start();
    pub fn section_text_end();
    pub fn section_rodata_end();
    pub fn section_data_end();
    pub fn section_bss_end();
    pub fn stack_low();
    pub fn stack_high();
    pub fn common_bss_sep();
}

/// Return the address of external symbols
pub fn get_ext_symb_add(f: unsafe extern "C" fn()) -> usize {
    f as *const usize as usize
}
