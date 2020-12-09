mod elf_sections;

#[repr(C)]
struct BootInfoHeader {
    total_size: u32,
    _reserved: u32,
}

#[repr(C)]
struct Tag {
    typ: u32,
    size: u32,
}

use core::marker::PhantomData;

struct TagIter<'a> {
    pub current: *const Tag,
    phantom: PhantomData<&'a Tag>,
}

impl<'a> TagIter<'a> {
    fn new(first: *const Tag) -> Self {
        TagIter {
            current: first,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        match unsafe{&*self.current} {
            &Tag{typ:0, size:8} => None, // end tag
            tag => {
                // go to next tag
                let mut tag_addr = self.current as usize;
                tag_addr += ((tag.size + 7) & !7) as usize; //align at 8 byte
                self.current = tag_addr as *const _;

                Some(tag)
            },
        }
    }
}

pub struct BootInformation {
    boot_info_header: *const BootInfoHeader,
}

impl BootInformation {
    /// Search for the ELF Sections tag.
    fn elf_sections_tag(&self) -> Option<elf_sections::ElfSectionsTag> {
        self.get_tag(9).map(|tag| elf_sections::ElfSectionsTag::new( unsafe {
                (tag as *const Tag).offset(1) as *const _ }))
    }
    fn get_tag<'a>(&'a self, typ: u32) -> Option<&'a Tag> {
        self.tags().find(|tag| tag.typ == typ)
    }

    fn tags(&self) -> TagIter {
        TagIter::new(unsafe { self.boot_info_header.offset(1) } as *const _)
    }
}

pub fn explore(multiboot2_info: usize) {
    let boot_info = BootInformation {
	boot_info_header: multiboot2_info as *const BootInfoHeader
    };
    let elf_sections_tag = boot_info.elf_sections_tag();
    match elf_sections_tag {
	Some(x) => {
	    let mut c = 0u32;
	    for section in  x.sections() {
		let zero = unsafe {section.string_table()} as usize;
		println!("{}: {} {:#x}\n",
		c, section.get(), zero);
		c += 1;
		if section.get().typ == 3 {
			println!("great");
			break;
		}
	    }
	    println!("{:?}", x.get());
	},
	None => (),
    }
}
