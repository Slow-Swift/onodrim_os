set -e

cd bootloader
cargo build
cd ../kernel
cargo build
cd ..

BOOTLOADER=$(pwd)/target/x86_64-unknown-uefi/debug/bootloader.efi
KERNEL=$(pwd)/target/x86_64-kernel/debug/kernel

mkdir -p esp/EFI/BOOT
mkdir -p esp/kernel
cp $BOOTLOADER esp/EFI/BOOT/BOOTX64.EFI
cp $KERNEL esp/kernel/kernel.elf