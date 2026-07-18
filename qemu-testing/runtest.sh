#!/bin/bash
#

pushd `pwd`
cd $(dirname $0)

if [[ $1 == "debug"]]; then
    QEMU_APPEND1=-s
    QEMU_APPEND2=-S
else
    QEMU_APPEND1=
    QEMU_APPEND2=
fi

cp ../target/x86_64-unknown-uefi/debug/uefiloader.efi esp/EFI/BOOT/BOOTX64.EFI

exec qemu-system-x86_64 -enable-kvm -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp \
    -serial stdio \
    $QEMU_APPEND1 $QEMU_APPEND2

popd
