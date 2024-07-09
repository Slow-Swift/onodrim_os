#!/bin/sh
set -e

# Create the GPT, create a 9MB partition starting at 1MB, and set the
# partition type to EFI System.
sgdisk \
    --clear \
    --new=1:1M:10M \
    --typecode=1:C12A7328-F81F-11D2-BA4B-00A0C93EC93B \
    /dev/sdb

# Format the partition as FAT.
mkfs.fat /dev/sdb1

# Mount the partition.
mkdir mount
mount /dev/sdb1 mount

# Copy the files to the partition
cp -r esp/EFI mount
cp -r esp/kernel mount

# Unmount the usb
umount mount
rmdir mount