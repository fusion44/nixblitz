---
slug: /
sidebar_position: 1
sidebar_label: Introduction
---

<div style={{textAlign: 'center'}}>
  <img src="img/logo.png" alt="nixblitz cheap ai slop image" style={{maxWidth: '30%'}} />
  <p style={{fontSize: '0.9em', color: '#555'}}>Mandatory nixblitz AI slop logo</p>
</div>

# Intoduction

NixBlitz is an experimental new version of the RaspiBlitz project that is currently being developed and tested. Consequently, it only supports a limited number of apps and features at present.

#### Tech stack:

- [Nix](https://nix.dev/manual/nix/): A package manager and build system for reproducibility
- [NixOS](https://nixos.org/): A Linux distribution that enables declarative and reproducible system configurations
- [nix-bitcoin](https://nixbitcoin.org/): A curated collection of Nix packages and NixOS modules designed to securely deploy Bitcoin nodes
- [Rust-lang](https://www.rust-lang.org/) For pretty much everything else (CLI and TUI, simple WebApp, etc.)

:::info
The NixBlitz Project is currently in the early stages of testing and research, so it may undergo significant changes. This means that configurations generated via the CLI may become incompatible at some point. Do not use it with mainnet funds.

Nix-bitcoin only supports `Mainnet` and `Regtest` at the moment, so does nixblitz.
:::

## Disclaimer

:::caution
This is Open-Source Software licensed under the MIT License. This license explicitly excludes the authors & publishers from any legal liabilities including funds you manage with this software. Its use at your own risk - see LICENSE legal text for details.

Also the RaspiBlitz offers lots of additional apps for install. With every additional app installed (or pre-installed in a fatpack sd card image) you are trusting also the authors & dependencies of those additional projects with the security of your system & funds (different legal licensed may apply also). To reduce pre-installed apps & features from the start we provide a minimal sd card image for more advanced users (see download section below). For more details on this topic see our SECURITY documentation.
:::

## Hardware Requirements

Currently, only X86 on bare metal and VMs are supported. Installing an ARM on a Raspberry Pi might work with some modifications, but this has not been tested and is not being actively developed.

Recommended Hardware:

- x86_64 CPU with 4 Cores
- 16GB RAM
- 2TB SSD (1TB will probably be not enough in the foreseeable future)

Currently, only systems with single disks are supported. A more flexible system will be supported in future.

## Useful links

- Repo - https://github.com/fusion44/nixblitz
- Docs - https://fusion44.github.io/nixblitz/
- Development chat
  - Signal: https://signal.group/#CjQKIGG9LdF6UkJjYODuPAwbUWYwVMmBcbdutWJpSJenJGIWEhCiqGUMg29Dku5o61jZvEak
  - Matrix: https://matrix.to/#/#nixblitz:matrix.org
