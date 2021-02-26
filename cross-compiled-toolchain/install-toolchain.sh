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
BINUTILS="binutils-2.35"
cd $SRC
if [ ! -d $BINUTILS ]; then
	echo ""
	echo "Installing \`binutils\`"
	echo ""
	curl http://ftp.gnu.org/gnu/binutils/$BINUTILS.tar.gz > $BINUTILS.tar.gz
	tar xfz $BINUTILS.tar.gz
	rm $BINUTILS.tar.gz
fi

if [ ! -f "$PREFIX/bin/i686-elf-ld" ]; then
	mkdir -p build-binutils
	cd build-binutils
	../$BINUTILS/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
	make
	make install
fi

echo "binutils OK"

# gcc
cd $SRC
GCC="gcc-10.2.0"
if [ ! -d $GCC ]; then
	echo ""
	echo "Installing \`gcc\`"
	echo ""
	curl -L http://ftpmirror.gnu.org/gcc/$GCC/$GCC.tar.gz > $GCC.tar.gz
	tar xvf $GCC.tar.gz
	rm $GCC.tar.gz
	mkdir -p build-gcc
fi

if [ ! -f "$PREFIX/bin/i686-elf-$GCC" ]; then
	cd $GCC
	sh ./contrib/download_prerequisites
	cd ../build-gcc
	../$GCC/configure --target="$TARGET" --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers

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

