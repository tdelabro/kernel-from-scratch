ARCH = i686
OS = kfs
TARGET = $(ARCH)-$(OS)
OPT_DIR = opt/

# Sources
SRC_DIR = src/

ASM_DIR = $(addprefix $(SRC_DIR), asm/)
ASM = multiboot_header.asm
SRC = $(addprefix $(ASM_DIR), ASM)

OTHERS_DIR = $(addprefix $(SRC_DIR), others/)
GRUB_CFG = $(addprefix $(OTHERS_DIR), grub.cfg)
LINKER = $(addprefix $(OTHERS_DIR), linker.ld)

# Builds
BUILD_DIR = build/
OBJ_DIR = $(addprefix $(BUILD_DIR), obj/)
ISO_DIR = $(addprefix $(BUILD_DIR), isodir/)
BOOT_DIR = $(addprefix $(ISO_DIR), boot/)
GRUB_DIR = $(addprefix $(BOOT_DIR), grub/)

OBJ = $(addprefix $(OBJ_DIR), $(ASM:.asm=.o))
KERNEL = $(addprefix $(BUILD_DIR), kernel.bin)
ISO = os.iso

# Rust
LIB = target/$(TARGET)/release/lib$(OS).a

default: $(ISO)

setup: $(OPT_DIR)
	rustup toolchain install nightly
	rustup component add rust-src
	sudo apt install xorriso
	sudo apt install qemu
	cargo install rustfilt

$(OPT_DIR):
	tar -Jxvf cross-compiler.tar.xz

$(BUILD_DIR):
	mkdir -p build

$(OBJ_DIR):
	mkdir -p build/obj

$(OBJ_DIR)%.o: $(ASM_DIR)%.asm
	nasm -f elf32 $< -o $@

lib:
	cargo build --release --target $(TARGET).json

$(KERNEL): $(OPT_DIR) $(OBJ_DIR) $(OBJ) lib
	opt/bin/i686-elf-gcc -T $(LINKER) -o $@ -ffreestanding -fno-builtin -fno-stack-protector -fno-omit-frame-pointer -fno-rtti -nostdlib -nodefaultlibs $(OBJ) $(LIB) -lgcc

$(ISO): $(BUILD_DIR) $(KERNEL)
	mkdir -p $(GRUB_DIR)
	cp $(GRUB_CFG) $(GRUB_DIR)
	cp $(KERNEL) $(BOOT_DIR)
	grub-mkrescue -o $@ $(ISO_DIR)

kernel: $(KERNEL)

iso: $(ISO)

doc:
	cargo doc --target $(TARGET).json

run: build
	qemu-system-i386 -cdrom $(ISO)

clean:
	rm -rf $(BUILD_DIR)
	cargo clean

fclean: clean
	rm -rf opt
	rm $(ISO)

re: clean default

.PHONY: setup default kernel iso doc run clean fclean re
