SRC_DIR = src/
LINKER_DIR = linker/
BUILD_DIR = build/
OPT_BIN_DIR = opt/bin/
OBJ_DIR = $(addprefix $(BUILD_DIR), obj/)
ISO_DIR = $(addprefix $(BUILD_DIR), isodir/)
BOOT_DIR = $(addprefix $(ISO_DIR), boot/)
GRUB_DIR = $(addprefix $(BOOT_DIR), grub/)

ASM = multiboot_header.asm boot.asm
KERNEL = $(addprefix $(BUILD_DIR), kernel.bin)
ISO = os.iso
LINKER = $(addprefix $(LINKER_DIR), linker.ld)
SRC = $(addprefix $(SRC_DIR), ASM)
OBJ = $(addprefix $(OBJ_DIR), $(ASM:.asm=.o))
GRUB_CFG = $(addprefix $(SRC_DIR), grub.cfg)

build: $(BUILD_DIR) $(ISO)

$(BUILD_DIR):
	mkdir -p build

$(OBJ_DIR):
	mkdir -p build/obj

$(OBJ_DIR)%.o: $(SRC_DIR)%.asm
	nasm -f elf32 $< -o $@

$(KERNEL): $(OBJ_DIR) $(OBJ) $(LINKER)
	$(OPT_BIN_DIR)i686-elf-gcc -T $(LINKER) -o $@ -ffreestanding -O2 -nostdlib $(OBJ) -lgcc

$(ISO): $(KERNEL) $(GRUB_CFG)
	mkdir -p $(GRUB_DIR)
	cp $(GRUB_CFG) $(GRUB_DIR)
	cp $(KERNEL) $(BOOT_DIR)
	grub-mkrescue -o $@ $(ISO_DIR)


run: build
	qemu-system-i386 -cdrom $(ISO)

clean:
	rm -rf $(BUILD_DIR)

fclean: clean
	rm $(ISO)

re: fclean build

.PHONY: build run clean fclean re
