---
sidebar_position: 1
sidebar_label: Setup a dev environment
---

# Development Environment Setup

NixBlitz relies on Nix Flakes for reproducible development environments. While using NixOS as a host system is recommended, other operating systems are supported.

## Prerequisites

Before setting up your development environment, ensure that Nix is installed on your host system. Follow the [official installation guide](https://nixos.org/download/#nix-install-linux) to get started. To effectively work with Nix Flakes, you need to enable two experimental features: nix-command and flakes. Depending on your system configuration, follow these steps:

### Configure Nix for development

To effectively work with the system, the experimental features `nix-command` and `flakes` must be enabled. Depending on your system and how Nix is installed the following steps must be taken:

#### 1. Allow Features on a Case-by-Case Basis

Add the following options to every Nix command:

```bash
nix --extra-experimental-features nix-command --extra-experimental-features flakes develop
```

#### 2. Enable Features Generally in the Config File

Edit `/etc/nix/nix.conf` and add the line:

```conf
experimental-features = nix-command flakes
```

#### 3. Configure NixOS (Declarative System Configuration)

If you're using NixOS, add this to your configuration file:

```nix
nix.extraOptions = "experimental-features = nix-command flakes";
```

### Enable Cross Compilation

To enable cross compilation of images and ISOs from the host system, ensure that QEMU is enabled. This process is very time consuming. Ways of how to increase cross compilation speed are [currently](https://www.cachix.org/) [being](https://github.com/zhaofengli/attic) investigated.

#### On NixOS

Add this to your NixOS configuration to enable `binfmt` emulation of `aarch64-linux`:

```nix
  boot.binfmt.emulatedSystems = ["aarch64-linux"];
```

Troubleshooting tips: https://github.com/nix-community/nixos-generators/tree/master?tab=readme-ov-file#cross-compiling

#### On Debian / Ubuntu

Follow the setup steps in the guide: https://wiki.debian.org/QemuUserEmulation
A system restart is required to make cross compilation work.
