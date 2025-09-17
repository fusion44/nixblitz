---
slug: /system-management
sidebar_position: 4
sidebar_label: System Management
---


import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# System Management

## Logging in via SSH

At this point SSH daemon is running and can be used to log into the nixblitz system:

```bash
# with default port
ssh admin@192.168.0.101

# with custom port (when running in a VM, etc)
ssh admin@192.168.0.101 -p 2222
```

Due to the bug described below, the password will be `nixblitz` until manually changed.

## Changing the password

Currently, there is a bug that the password is not applied correctly during the installation. It is recommended to change the password after the installation using the `passwd` command.

## The `nixblitz` CLI tool

Most system management operations are done via the `nixblitz` command:

```bash
~> nixblitz --help
A CLI interface to the RaspiBlitz project.

Usage: nixblitz [OPTIONS] [COMMAND]

Commands:
  set      Sets an app option in the given configuration
  tui      Opens the TUI in the given work dir
  init     Initializes a new project in the given work dir
  install  Installs the system defined in the given work dir
  apply    Applies changes to the system defined in the given work dir
  doctor   Analyze the project for common problems
  help     Print this message or the help of the given subcommand(s)

Options:
      --log-level <LOG_LEVEL>  Set the log level (overrides environment variable and config file)
      --log-file <LOG_FILE>    Set the log file path (overrides environment variable and default)
  -h, --help                   Print help
  -V, --version                Print version
```

Important commands for an installed system `tui` and `apply`.

:::warning
Do not reinstall the system. This will wipe the drive and install with a fresh configuration. This command will be hidden on installed systems for future releases.
:::

### Debugging the CLI tool

The CLI tool logs to a file in `~/.local/state/nixblitz-cli/nixblitz.log` with log level `INFO` by default.

#### Change the log level

The log level can be changed by setting the `NIXBLITZ_LOG_LEVEL` environment variable to one of the following values: `["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"]`

Set the log level for the duration of the current shell session:
<Tabs groupId="shell-type">
<TabItem value="bash" label="bash">

```bash
export NIXBLITZ_LOG_LEVEL="TRACE"
```

</TabItem>
<TabItem value="nushell" label="nu" default>

```nu
$env.NIXBLITZ_LOG_LEVEL = "TRACE"
```

</TabItem>
</Tabs>

Subsequent calls to `nixblitz apply` will use the new log level until the shell session ends.

Passing the `--log-level` flag to the `nixblitz` command will apply the log level change to the current command only: `nixblitz --log-level=TRACE apply`. This will override the `NIXBLITZ_LOG_LEVEL` environment variable.

#### View the log file

```bash
# or use `cat` if preferred
> bat ~/.local/state/nixblitz-cli/nixblitz.log
───────┬────────────────────────────────────────────────────────────────────────────
       │ File: .local/state/nixblitz-cli/nixblitz.log
───────┼────────────────────────────────────────────────────────────────────────────
   1   │ 2025-06-07 08:09:47 [INFO ]
   2   │ ===========================================================================
   3   │ Initialized logging
   4   │   Level: INFO
   5   │   File:  /home/admin/.local/state/nixblitz-cli/nixblitz.log
   6   │   Time:  2025-06-07 08:09:47
   7   │ ===========================================================================
   8   │ 2025-06-07 08:18:28 [INFO ]
   9   │ ===========================================================================
  10   │ Initialized logging
  11   │   Level: INFO
  12   │   File:  /home/admin/.local/state/nixblitz-cli/nixblitz.log
  13   │   Time:  2025-06-07 08:18:28
  14   │ ===========================================================================
  15   │ 2025-06-07 08:18:28 [INFO ] Starting NixBlitz installation wizard
  16   │ 2025-06-07 08:18:28 [INFO ] Working directory: "/mnt/data/config"
  17   │ 2025-06-07 08:18:28 [INFO ] Step 0: Requesting installation target selection (local/remote)
  18   │ 2025-06-07 08:18:40 [INFO ] User selected local installation target
  19   │ 2025-06-07 08:18:40 [INFO ] Proceeding with local installation
  ...
```

## Configuration files

The configuration files are located in `/mnt/data/config/`.

```
~> ls /mnt/data/config/
╭───┬──────────────────────────────┬──────┬─────────┬──────────────╮
│ # │             name             │ type │  size   │   modified   │
├───┼──────────────────────────────┼──────┼─────────┼──────────────┤
│ 0 │ /mnt/data/config/dev_scripts │ dir  │ 4.09 kB │ 17 hours ago │
│ 1 │ /mnt/data/config/flake.lock  │ file │ 1.49 kB │ 17 hours ago │
│ 2 │ /mnt/data/config/flake.nix   │ file │   564 B │ 17 hours ago │
│ 3 │ /mnt/data/config/justfile    │ file │ 1.05 kB │ 17 hours ago │
│ 4 │ /mnt/data/config/src         │ dir  │ 4.09 kB │ 17 hours ago │
╰───┴──────────────────────────────┴──────┴─────────┴──────────────╯
```

These files store the system's configuration. Editing these files manually is not recommended, as the CLI might not be able to handle the changes.

<details>
    <summary>Manual Configuration</summary>

The configuration can be managed manually from this point on, but experience with Nix and NixOS is recommended.
The actual configuration is located in `/mnt/data/config/src/configuration.common.nix`. There are several subfolders with specialized configuration files for various platforms.

</details>

<details>
    <summary>Possibly useful info about the configuration files</summary>

    The configuration files are a git repository. The cli creates a new commit when changes are applied. The commits can be manually pushed to a remote repository.
    ```bash
    /mnt/data/config/src> git log

    commit b2e60b3de1e686297ab3dbf2c5bcb10119263f55 (HEAD -> main)
    Author: nixblitz <nixblitz>
    Date:   Sat Jun 7 08:23:05 2025 -0400

        update config

    commit 8b71372ad678f8224f13e050e965ae25ac2d10ee
    Author: nixblitz <nixblitz>
    Date:   Fri Jun 6 20:28:24 2025 +0200

        system installed

    commit b60ef45ad85f3b55e42f6dc56495b9b68474c661
    Author: nixblitz <nixblitz>
    Date:   Fri Jun 6 20:25:53 2025 +0200

        init
    ```

</details>

## Changing Settings

:::info
Changing the settings does not apply the changes immediately. This is done in the next step via `nixblitz apply`.
:::

### Changing via the WebApp

The web app to change settings is served at the root path of the device's IP address. E.g.: `http://192.168.8.123/` Any changes made are saved automatically.

:::warning
The web app is currently not secured. Anyone in the network can access it and change the settings, but not apply them.
:::

### Changing via the TUI

The TUI can be started with `nixblitz tui`.

## Applying changes

The `apply` command applies the changes made to the current system config:

```bash
doas nixblitz apply
```

Running the command without `doas` (sudo alternative) will fail with a permission denied error:

```bash
...
error: creating symlink '/nix/var/nix/profiles/system-2-link.tmp-3952-130241961' -> '/nix/store/59rcn6ssqmgcr5nv46g4nazhisf2mnqm-nixos-system-nixblitzvm-25.11.20250603.c2a0396': Permission denied
--- Process finished with status: exit status: 1 ---
Error applying changes: Unknown error
├╴at cli/src/commands/apply.rs:12:10
│
╰─▶ Unable to apply changes
    ├╴at /build/packages/nixblitzlib/src/utils.rs:254:33
    ╰╴Command exited with non-zero status.
```

:::info
This command internally uses `nixos-rebuild switch` to apply the changes.
:::

## Accessing Bitcoin, LND etc.

When logged in via SSH, the `bitcoin-cli` and `lncli` commands are available in the `admin` user's home directory.

```bash
~> bitcoin-cli --getinfo
Chain: regtest
Blocks: 0
Headers: 0
Verification progress: 100.0000%
Difficulty: 4.656542373906925e-10

Network: in 0, out 0, total 0
Version: 290000
Time offset (s): 0
Proxies: 127.0.0.1:9050 (ipv4, ipv6, onion, cjdns)
Min tx relay fee rate (BTC/kvB): 0.00001000
```

```bash
~> lncli getinfo
{
    "version": "0.18.5-beta commit=",
    "commit_hash": "",
    "identity_pubkey": "03e15b5df6bce41c0333865f6b204c9eee3b467c4e3a0416790b18c897201ce95f",
    "alias": "03e15b5df6bce41c0333",
    "color": "#3399ff",
    "num_pending_channels": 0,
    "num_active_channels": 0,
    "num_inactive_channels": 0,
    "num_peers": 0,
    ...
    "testnet": false,
    "chains": [
        {
            "chain": "bitcoin",
            "network": "regtest"
        }
    ],
    ...
}
```
