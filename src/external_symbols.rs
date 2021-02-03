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

unsafe fn get_ext_symb_add(f: unsafe extern "C" fn()) -> usize {
    f as *const usize as usize
}

pub fn get_kernel_start() -> usize {
    unsafe { get_ext_symb_add(kernel_start) }
}
pub fn get_kernel_end() -> usize {
    unsafe { get_ext_symb_add(kernel_end) }
}
pub fn get_section_text_start() -> usize {
    unsafe { get_ext_symb_add(section_text_start) }
}
pub fn get_section_rodata_start() -> usize {
    unsafe { get_ext_symb_add(section_rodata_start) }
}
pub fn get_section_data_start() -> usize {
    unsafe { get_ext_symb_add(section_data_start) }
}
pub fn get_section_bss_start() -> usize {
    unsafe { get_ext_symb_add(section_bss_start) }
}
pub fn get_section_text_end() -> usize {
    unsafe { get_ext_symb_add(section_text_end) }
}
pub fn get_section_rodata_end() -> usize {
    unsafe { get_ext_symb_add(section_rodata_end) }
}
pub fn get_section_data_end() -> usize {
    unsafe { get_ext_symb_add(section_data_end) }
}
pub fn get_section_bss_end() -> usize {
    unsafe { get_ext_symb_add(section_bss_end) }
}
pub fn get_stack_low() -> usize {
    unsafe { get_ext_symb_add(stack_low) }
}
pub fn get_stack_high() -> usize {
    unsafe { get_ext_symb_add(stack_high) }
}
pub fn get_common_bss_sep() -> usize {
    unsafe { get_ext_symb_add(common_bss_sep) }
}
