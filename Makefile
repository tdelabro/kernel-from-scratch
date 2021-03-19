ARCH = i686
OS = kfs
RAM_AMOUNT=128M
TARGET = $(ARCH)-$(OS)
OPT_DIR = cross-compiled-toolchain/opt/

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

CC = $(addprefix $(OPT_DIR), bin/i686-elf-gcc)
ifeq ($(shell uname), Darwin)
	GRUB_MKRESCUE = $(addprefix $(OPT_DIR), bin/grub-mkrescue)
else
	GRUB_MKRESCUE = grub-mkrescue
endif


OBJ = $(addprefix $(OBJ_DIR), $(ASM:.asm=.o))
KERNEL = $(addprefix $(BUILD_DIR), kernel.bin)
ISO = os.iso

# Rust
LIB = target/$(TARGET)/release/lib$(OS).a

default: $(ISO)

setup: $(OPT_DIR)
	rustup toolchain install nightly
	rustup component add rust-src
	cd cross-compiled-toolchain && sh install-toolchain.sh

$(OPT_DIR):
	#mkdir $(OPT_DIR)

$(BUILD_DIR):
	mkdir -p build

$(OBJ_DIR):
	mkdir -p build/obj

$(OBJ_DIR)%.o: $(ASM_DIR)%.asm
	nasm -f elf32 $< -o $@

lib:
	cargo build --release --target $(TARGET).json

$(KERNEL): $(OPT_DIR) $(OBJ_DIR) $(OBJ) lib
	$(CC) -T $(LINKER) -o $@ -ffreestanding -fno-builtin -fno-stack-protector -fno-omit-frame-pointer -fno-rtti -nostdlib -nodefaultlibs $(OBJ) $(LIB) -lgcc

$(ISO): $(BUILD_DIR) $(KERNEL)
	mkdir -p $(GRUB_DIR)
	cp $(GRUB_CFG) $(GRUB_DIR)
	cp $(KERNEL) $(BOOT_DIR)
	$(GRUB_MKRESCUE) -o $@ $(ISO_DIR)

kernel: $(KERNEL)

iso: $(ISO)

doc:
	cargo doc --target $(TARGET).json

run: $(ISO)
	qemu-system-i386 -m 128M -cdrom $(ISO) -m $(RAM_AMOUNT) -vga std

clean:
	rm -rf $(BUILD_DIR)
	cargo clean

fclean: clean
	rm -rf opt
	rm $(ISO)

re: clean default

.PHONY: setup default kernel iso doc run clean fclean re
