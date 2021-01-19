#!bin/sh

brew install gmp mpfr libmpc autoconf automake nasm xorriso qemu

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
	mkdir -p build-binutils
	cd build-binutils
	../binutils-2.35/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
	make
	make install
fi

# gcc
cd "$SRC"
if [ ! -d "gcc-10.2.0" ]; then
	echo ""
	echo "Installing \`gcc\`"
	echo ""
	curl -L http://ftpmirror.gnu.org/gcc/gcc-10.2.0/gcc-10.2.0.tar.gz > gcc-10.2.0.tar.gz
	tar jxf gcc-10.2.0.tar.gz
	rm gcc-10.2.0.tar.gz
	mkdir -p build-gcc
	cd build-gcc
	../gcc-10.2.0/configure --target="$TARGET" --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers --with-gmp="$(brew --prefix gmp)" --with-mpfr="$(brew --prefix mpfr)" --with-mpc="$(brew --prefix libmpc)"
	make all-gcc
	make all-target-libgcc
	make install-gcc
	make install-target-libgcc
fi

# grub
cd "$HOME/src"
if [ ! -d "grub" ]; then
	echo ""
	echo "Installing \`grub\`"
	echo ""
	git clone --depth 1 git://git.savannah.gnu.org/grub.git
	cd grub
	./bootstrap
	mkdir -p build-grub
	cd build-grub
	../configure --disable-werror TARGET_CC=$TARGET-gcc TARGET_OBJCOPY=$TARGET-objcopy TARGET_STRIP=$TARGET-strip TARGET_NM=$TARGET-nm TARGET_RANLIB=$TARGET-ranlib --target=$TARGET --prefix="$PREFIX"
	make
	make install
fi
