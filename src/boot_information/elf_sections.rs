#[derive(Debug)]
#[repr(C, packed)]
pub struct ElfSectionsTagInner {
    pub number_of_sections: u32,
    pub entry_size: u32,
    pub shndx: u32, // string table
}

#[derive(Debug)]
pub struct ElfSectionsTag {
    inner: *const ElfSectionsTagInner, 
}

impl ElfSectionsTag {
    pub fn new(elf_tag: *const usize) -> ElfSectionsTag {
        ElfSectionsTag { inner: elf_tag as *const ElfSectionsTagInner }
    }

    pub fn first_section(&self) -> *const ElfSectionInner {
        (unsafe { self.inner.offset(1) }) as *const _
    }

    pub fn sections(&self) -> ElfSectionIter {
        let string_section_offset = 
	    (self.get_shndx() * self.get_entry_size()) as isize;
	let string_section_ptr = unsafe {
	    self.first_section().offset(string_section_offset) as *const _
	};
	ElfSectionIter {
	    current_section: self.first_section(),
	    remaining_sections: self.get().number_of_sections,
	    entry_size: self.get().entry_size,
	    string_section: string_section_ptr,
	}
    }

    pub fn get(&self) -> &ElfSectionsTagInner {
	unsafe { &*self.inner }
    }

    pub fn get_number_of_sections(&self) -> u32 {
	self.get().number_of_sections
    }
    pub fn get_entry_size(&self) -> u32 {
	self.get().entry_size
    }
    pub fn get_shndx(&self) -> u32 {
	self.get().shndx
    }
}

/// An iterator over some ELF sections.
#[derive(Clone, Debug)]
pub struct ElfSectionIter {
    current_section: *const ElfSectionInner,
    remaining_sections: u32,
    entry_size: u32,
    string_section: *const ElfSectionInner,
}

impl Iterator for ElfSectionIter {
    type Item = ElfSection;

    fn next(&mut self) -> Option<ElfSection> {
	while self.remaining_sections != 0 {
	    let section = ElfSection {
		inner: self.current_section,
		string_section: self.string_section,
	    };

	    self.current_section = unsafe {
		self.current_section.offset(self.entry_size as isize)
	    };
	    self.remaining_sections -= 1;

	    if section.section_type() != ElfSectionType::Unused {
		return Some(section);
	    }
	}
	None
    }
}

/// A single generic ELF Section.
#[derive(Debug)]
pub struct ElfSection {
    inner: *const ElfSectionInner,
    string_section: *const ElfSectionInner,
}

impl ElfSection {
    /// Get the section type as a `ElfSectionType` enum variant.
    pub fn section_type(&self) -> ElfSectionType {
	match self.get().typ {
	    0 => ElfSectionType::Unused,
	    1 => ElfSectionType::ProgramSection,
	    2 => ElfSectionType::LinkerSymbolTable,
	    3 => ElfSectionType::StringTable,
	    4 => ElfSectionType::RelaRelocation,
	    5 => ElfSectionType::SymbolHashTable,
	    6 => ElfSectionType::DynamicLinkingTable,
	    7 => ElfSectionType::Note,
	    8 => ElfSectionType::Uninitialized,
	    9 => ElfSectionType::RelRelocation,
	    10 => ElfSectionType::Reserved,
	    11 => ElfSectionType::DynamicLoaderSymbolTable,
	    0x6000_0000..=0x6FFF_FFFF => ElfSectionType::EnvironmentSpecific,
	    0x7000_0000..=0x7FFF_FFFF => ElfSectionType::ProcessorSpecific,
	    _ => panic!(),
	}
    }

    /// Read the name of the section.
    pub fn name(&self) -> &str {
	use core::{slice, str};

	let name_ptr = unsafe {
	    self.string_table().offset(self.get().name_index as isize)
	};
	let strlen = {
	    let mut len = 0;
	    while unsafe { *name_ptr.offset(len) } != 0 {
		len += 1;
	    }
	    len as usize
	};

	match str::from_utf8(unsafe { slice::from_raw_parts(name_ptr, strlen) }) {
	    Ok(s) => s,
	    Err(_) => "err",
	}
    }

    pub fn get(&self) -> &ElfSectionInner {
	unsafe { &*self.inner }
    }

    pub unsafe fn string_table(&self) -> *const u8 {
	(*self.string_section).addr as *const _
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct ElfSectionInner {
    name_index: u32,
    pub typ: u32,
    flags: u32,
    addr: u32,
    pub offset: u32,
    pub size: u32,
    link: u32,
    info: u32,
    addralign: u32,
    entry_size: u32,
}

use core::fmt;

impl fmt::Display for ElfSectionInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "name_idx: {:#x}, type: {:#x}, flags: {:#x}, addr: {:#x}, offset: {:#x}, size: {:#x}, link: {:#x}, info: {:#x}, addralign: {:#x}, entry_size: {:#x})",
	self.name_index, self.typ, self.flags, self.addr, self.offset, self.size,
	self.link, self.info, self.addralign, self.entry_size)
    }
} 

/// An enum abstraction over raw ELF section types.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(u32)]
pub enum ElfSectionType {
    /// This value marks the section header as inactive; it does not have an
    /// associated section. Other members of the section header have undefined
    /// values.
    Unused = 0,

    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    ProgramSection = 1,

    /// This section holds a linker symbol table.
    LinkerSymbolTable = 2,

    /// The section holds a string table.
    StringTable = 3,

    /// The section holds relocation entries with explicit addends, such as type
    /// Elf32_Rela for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelaRelocation = 4,

    /// The section holds a symbol hash table.
    SymbolHashTable = 5,

    /// The section holds dynamic linking tables.
    DynamicLinkingTable = 6,

    /// This section holds information that marks the file in some way.
    Note = 7,

    /// A section of this type occupies no space in the file but otherwise resembles
    /// `ProgramSection`. Although this section contains no bytes, the
    /// sh_offset member contains the conceptual file offset.
    Uninitialized = 8,

    /// The section holds relocation entries without explicit addends, such as type
    /// Elf32_Rel for the 32-bit class of object files. An object file may have
    /// multiple relocation sections.
    RelRelocation = 9,

    /// This section type is reserved but has unspecified semantics.
    Reserved = 10,

    /// This section holds a dynamic loader symbol table.
    DynamicLoaderSymbolTable = 11,

    /// Values in this inclusive range (`[0x6000_0000, 0x6FFF_FFFF)`) are
    /// reserved for environment-specific semantics.
    EnvironmentSpecific = 0x6000_0000,

    /// Values in this inclusive range (`[0x7000_0000, 0x7FFF_FFFF)`) are
    /// reserved for processor-specific semantics.
    ProcessorSpecific = 0x7000_0000,
}
