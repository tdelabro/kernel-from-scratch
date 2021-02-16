//! Symbols defined in the linker script
//! 
//! Provide information about kernel the memory layout.
//! Used for kernel identity mapping and debug.

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
    pub fn first_page_after_kernel();
}

const unsafe fn get_ext_symb_add(f: unsafe extern "C" fn()) -> *const usize {
    f as *const usize
}

pub fn get_kernel_start() -> *const usize {
    unsafe { get_ext_symb_add(kernel_start) }
}
pub fn get_kernel_end() -> *const usize {
    unsafe { get_ext_symb_add(kernel_end) }
}
pub const fn get_first_page_after_kernel() -> *const usize {
    unsafe { get_ext_symb_add(first_page_after_kernel) }
}
pub fn get_section_text_start() -> *const usize {
    unsafe { get_ext_symb_add(section_text_start) }
}
pub fn get_section_rodata_start() -> *const usize {
    unsafe { get_ext_symb_add(section_rodata_start) }
}
pub fn get_section_data_start() -> *const usize {
    unsafe { get_ext_symb_add(section_data_start) }
}
pub fn get_section_bss_start() -> *const usize {
    unsafe { get_ext_symb_add(section_bss_start) }
}
pub fn get_section_text_end() -> *const usize {
    unsafe { get_ext_symb_add(section_text_end) }
}
pub fn get_section_rodata_end() -> *const usize {
    unsafe { get_ext_symb_add(section_rodata_end) }
}
pub fn get_section_data_end() -> *const usize {
    unsafe { get_ext_symb_add(section_data_end) }
}
pub fn get_section_bss_end() -> *const usize {
    unsafe { get_ext_symb_add(section_bss_end) }
}
pub fn get_stack_low() -> *const usize {
    unsafe { get_ext_symb_add(stack_low) }
}
pub fn get_stack_high() -> *const usize {
    unsafe { get_ext_symb_add(stack_high) }
}
pub fn get_common_bss_sep() -> *const usize {
    unsafe { get_ext_symb_add(common_bss_sep) }
}
