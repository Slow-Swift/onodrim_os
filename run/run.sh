#!/bin/sh
set -e

run/build.sh

OVMF_CODE=$(pwd)/run/OVMF_CODE.fd
OVMF_VARS=$(pwd)/run/OVMF_VARS.fd

qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=$OVMF_CODE \
    -drive if=pflash,format=raw,readonly=on,file=$OVMF_VARS \
    -drive format=raw,file=fat:rw:esp \
    -serial mon:stdio