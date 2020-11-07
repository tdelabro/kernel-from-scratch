ARCH = i686
OS = kfs
TARGET = $(ARCH)-$(OS)

# Sources
SRC_DIR = src/

ASM_DIR = $(addprefix $(SRC_DIR), asm/)
ASM = multiboot_header.asm boot.asm
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

$(BUILD_DIR):
	mkdir -p build

$(OBJ_DIR):
	mkdir -p build/obj

$(OBJ_DIR)%.o: $(ASM_DIR)%.asm
	nasm -f elf32 $< -o $@

lib:
	cargo build --release --target $(TARGET).json

$(KERNEL): $(OBJ_DIR) $(OBJ) lib
	opt/bin/i686-elf-gcc -T $(LINKER) -o $@ -ffreestanding -fno-builtin -fno-stack-protector -fno-rtti -nostdlib -nodefaultlibs -O2 $(OBJ) $(LIB) -lgcc
#	 gcc -m32 -T $(LINKER) $(OBJ) $(LIB) -o $@ -fno-builtin -fno-stack-protector -fno-rtti -nostdlib -nodefaultlibs -ffreestanding 

$(ISO): $(BUILD_DIR) $(KERNEL)
	mkdir -p $(GRUB_DIR)
	cp $(GRUB_CFG) $(GRUB_DIR)
	cp $(KERNEL) $(BOOT_DIR)
	grub-mkrescue -o $@ $(ISO_DIR)

kernel: $(KERNEL)

iso: $(ISO)

run: build
	qemu-system-i386 -cdrom $(ISO)

clean:
	rm -rf $(BUILD_DIR)
	cargo clean

fclean: clean
	rm $(ISO)

re: fclean default

.PHONY: default kernel iso run clean fclean re
