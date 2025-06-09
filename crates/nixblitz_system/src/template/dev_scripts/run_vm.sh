#!/bin/bash

# Set the disk image path and size
DISK_PATH="./disk.img"
DISK_SIZE="30G"

cd src

# Check if the disk image exists
if [ ! -f "$DISK_PATH" ]; then
    echo "Creating disk image at $DISK_PATH..."
    # Create the disk image using qemu-img
    qemu-img create -f raw "$DISK_PATH" "$DISK_SIZE"
    if [ $? -eq 0 ]; then
        echo "Disk image created successfully"
    else
        echo "Error creating disk image"
        exit 1
    fi
else
    echo "Disk image already exists at $DISK_PATH"
fi

nixos-rebuild build-vm --flake .#nixblitzvm

export QEMU_OPTS="-drive file=${cwd}/disk.img,format=raw,index=0,media=disk"
export QEMU_NET_OPTS="hostfwd=tcp::18444-:18444,hostfwd=tcp::10022-:22,hostfwd=tcp::8080-:80,hostfwd=tcp::9735-:9735"
./result/bin/run-nixblitzvm-vm
