#!bin/sh

OS=$(uname -s)
case $OS in 
	Linux) sudo apt-get install autoconf automake nasm xorriso qemu;;
	Darwin) brew install autoconf automake nasm xorriso qemu;;
esac


export PREFIX="$PWD/opt"
export SRC="$PWD/src"
export TARGET=i686-elf
export PATH="$PREFIX/bin:$PATH"

mkdir -p "$PREFIX"
mkdir -p "$SRC"

# binutils
cd $SRC
if [ ! -d "binutils-2.35" ]; then
	echo ""
	echo "Installing \`binutils\`"
	echo ""
	curl http://ftp.gnu.org/gnu/binutils/binutils-2.35.tar.gz > binutils-2.35.tar.gz
	tar xfz binutils-2.35.tar.gz
	rm binutils-2.35.tar.gz
fi

if [ ! -f "$PREFIX/bin/i686-elf-ld" ]; then
	mkdir -p build-binutils
	cd build-binutils
	../binutils-2.35/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
	make
	make install
fi

echo "binutils OK"

# gcc
cd $SRC
if [ ! -d "gcc-10.2.0" ]; then
	echo ""
	echo "Installing \`gcc\`"
	echo ""
	curl -L http://ftpmirror.gnu.org/gcc/gcc-10.2.0/gcc-10.2.0.tar.gz > gcc-10.2.0.tar.gz
	tar xvf gcc-10.2.0.tar.gz
	rm gcc-10.2.0.tar.gz
	mkdir -p build-gcc
fi

if [ ! -f "$PREFIX/bin/i686-elf-gcc-10.2.0" ]; then
	cd gcc-10.2.0
	sh ./contrib/download_prerequisites
	cd ../build-gcc
	../gcc-10.2.0/configure --target="$TARGET" --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers

	make all-gcc
	make all-target-libgcc
	make install-gcc
	make install-target-libgcc
fi

echo "gcc OK"

# grub
if [ $OS = "Darwin" ]; then
	cd $SRC
	if [ ! -d "grub" ]; then
		echo ""
		echo "Installing \`grub\`"
		echo ""
		git clone --depth 1 git://git.savannah.gnu.org/grub.git
	fi
	cd grub
	./bootstrap
	mkdir -p build-grub
	cd build-grub
	../configure --disable-werror TARGET_CC=$TARGET-gcc TARGET_OBJCOPY=$TARGET-objcopy TARGET_STRIP=$TARGET-strip TARGET_NM=$TARGET-nm TARGET_RANLIB=$TARGET-ranlib --target=$TARGET --prefix="$PREFIX"
	make
	make install

	echo "grub OK"
fi

