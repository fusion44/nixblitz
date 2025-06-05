---
slug: /
sidebar_position: 1
---

# Intoduction

NixBlitz is an experimental new version of the RaspiBlitz project currently in development and testing. As such, it currently only has a limited number of supported apps and features.

Tech stack:

- [Nix](https://nix.dev/manual/nix/): A package manager and build system for reproducibility
- [NixOS](https://nixos.org/): A Linux distribution that enables declarative and reproducible system configurations
- [nix-bitcoin](https://nixbitcoin.org/): A curated collection of Nix packages and NixOS modules designed to securely deploy Bitcoin nodes
- [Rust-lang](https://www.rust-lang.org/) For the CLI app

:::info
The NixBlitz Project is currently in its early testing and research phase, and as such, it may undergo significant changes. This means that configurations generated via the CLI may become incompatible at some point. Don't use it on with mainnet funds.
:::

## Disclaimer

:::caution
This is Open-Source Software licensed under the MIT License. This license explicitly excludes the authors & publishers from any legal liabilities including funds you manage with this software. Its use at your own risk - see LICENSE legal text for details.

Also the RaspiBlitz offers lots of additional apps for install. With every additional app installed (or pre-installed in a fatpack sd card image) you are trusting also the authors & dependencies of those additional projects with the security of your system & funds (different legal licensed may apply also). To reduce pre-installed apps & features from the start we provide a minimal sd card image for more advanced users (see download section below). For more details on this topic see our SECURITY documentation.
:::

## Hardware Requirements

Currently we only support X86 on bare metal and VMs. Installing an a ARM on a Raspberry PI might work with some hackery, but it is not tested nor is it being actively developed.

Recommended Hardware:

- x86_64 CPU with 4 Cores
- 16GB RAM
- 2TB SSD (1TB will probably be not enough in the foreseeable future)

Currently only system with a single disk is supported. A more flexible system will be supported in the future.

## Useful links

- Repo - https://github.com/fusion44/nixblitz
- Docs - https://github.com/fusion44/nixblitz-docs
- Development chat - https://matrix.to/#/#nixblitz:matrix.org
