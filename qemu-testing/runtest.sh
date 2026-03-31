#!/bin/bash
#

pushd `pwd`
cd $(dirname $0)
rm -f ./esp/EFI/BOOT/BOOTX64.EFI
cp ../target/x86_64-unknown-uefi/debug/uefiloader.efi ./esp/EFI/BOOT/BOOTX64.EFI

exec qemu-system-x86_64 -machine q35 -accel kvm -accel tcg \
    -m 256 \
    -pflash OVMF_CODE.fd \
    -pflash OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:./esp \
    -debugcon file:uefi_debug.log -global isa-debugcon.iobase=0x402

popd