---
sidebar_position: 2
sidebar_label: Compiling images from configs
---

# Creating Flashable Images and ISOs with NixBlitz

NixBlitz enables you to create flashable images and ISOs on any host system for any target system supported by Nix. To explore the extensive list of supported systems, refer to the [Nixos Generators documentation](https://github.com/nix-community/nixos-generators)

## Prerequisites

This guide assumes that your host system is set up for image and ISO creation. For instructions on setting up your development environment, see our [dev environment setup category](dev_env.md).

:::warning
Initially building an SD card for a different system architecture can take a significant amount of time (e.g., 6-7 hours on an AMD Ryzen 7 7800X3D 8-Core Processor with 64GB RAM) due to the necessary QEMU emulation of the target system. For example, creating an image from an x86 host for AARCH64 (Raspberry PI) may take several hours. However, subsequent builds are much faster since the results are cached locally.
:::

## Useful links

- [nix build](https://nix.dev/manual/nix/latest/command-ref/new-cli/nix3-build)

## Creating Flashable Images
