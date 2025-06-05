# Known problems

- CLI/WebApp: the modified marker doesn't work for some options, the logic behind it is flawed

- WebApp: number input doesn't discern between integers and floats and silently fails

- There are currently two kinds of webapps, which are confusingly named:

  - Raspiblitz Web (normal Raspiblitz Web App)
  - NixBlitz Web App (Simple web app for managing the system)
    They are confusingly named, but they serve different purposes.

- Setting the password via the TUI / WebApp doesn't work => after first login (default pw: nixblitz) change it via the standard `passwd` command

- There might be problems setting IPV6 addresses via the CLI and the WebApp

- Too many options are exposed to the user. Maybe it's better to have an advanced mode, and by default only expose a limited number of options

- during development, the admin users shell is [nushell](https://www.nushell.sh/).

- `nixblitz install` is still available on an installed system => should be turned off

- When applying a config and the system builds successfully, but a service can't be started, then NixOS switched to the configuration but the new state is not committed to git automatically

- When applying a config on an INSTALLED system, we explicitly need to tell the cli where to find the config. This is because the env is sanitized when running sudo/doas.

- When enabling core lightning, the clightning service can not be started due to an error:

  ```
  May 18 09:00:02 nixblitzvm systemd[1]: clightning.service: start-post operation timed out. Terminating.
  May 18 09:00:02 nixblitzvm systemd[1]: clightning.service: Control process exited, code=killed, status=15/TERM
  May 18 09:00:02 nixblitzvm systemd[1]: clightning.service: Failed with result 'timeout'.
  May 18 09:00:02 nixblitzvm systemd[1]: Failed to start clightning.service.
  May 18 09:00:02 nixblitzvm systemd[1]: clightning.service: Consumed 1.645s CPU time, 51.2M memory peak.
  May 18 09:00:13 nixblitzvm systemd[1]: clightning.service: Scheduled restart job, restart counter is at 1.
  May 18 09:00:13 nixblitzvm systemd[1]: Starting clightning.service...
  May 18 09:00:13 nixblitzvm lightningd[17697]: INFO    lightningd: v25.02.1
  May 18 09:00:13 nixblitzvm lightningd[17697]: INFO    plugin-wss-proxy: Killing plugin: disabled itself: No python3 binary found
  May 18 09:00:13 nixblitzvm lightningd[17697]: UNUSUAL plugin-bookkeeper: topic 'utxo_deposit' is not a known notification topic
  May 18 09:00:13 nixblitzvm lightningd[17697]: UNUSUAL plugin-bookkeeper: topic 'utxo_spend' is not a known notification topic
  May 18 09:00:13 nixblitzvm lightningd[17697]: **BROKEN** lightningd: failed to open database /mnt/data/clightning/bitcoin/lightningd.sqlite3: unable to open database file
  May 18 09:00:13 nixblitzvm lightningd[17697]: failed to open database /mnt/data/clightning/bitcoin/lightningd.sqlite3: unable to open database file
  May 18 09:00:13 nixblitzvm systemd[1]: clightning.service: Main process exited, code=exited, status=1/FAILURE
  ```

- When you have a config with bitcoind and lnd or clighthning enabled and disable bitcoind without disabling lnd or clightning, the config will fail to apply:

  ```
  warning: Nix search path entry '/nix/var/nix/profiles/per-user/root/channels' does not exist, ignoring
  error:
       … while calling the 'head' builtin
         at /nix/store/x9wnkly3k1gkq580m90jjn32q9f05q2v-source/lib/attrsets.nix:1534:13:
         1533|           if length values == 1 || pred here (elemAt values 1) (head values) then
         1534|             head values
             |             ^
         1535|           else

       … while evaluating the attribute 'value'
         at /nix/store/x9wnkly3k1gkq580m90jjn32q9f05q2v-source/lib/modules.nix:1084:7:
         1083|     // {
         1084|       value = addErrorContext "while evaluating the option `${showOption loc}':" value;
             |       ^
         1085|       inherit (res.defsFinal') highestPrio;

       … while evaluating the option `system.build.toplevel':

       … while evaluating definitions from `/nix/store/x9wnkly3k1gkq580m90jjn32q9f05q2v-source/nixos/modules/system/activation/top-level.nix':

       … while evaluating the option `assertions':

       … while evaluating definitions from `/nix/store/3ipnrh8kacbhawf90zsi5y0i8kpv7b1b-source/nixos/common.nix':

       … while evaluating the option `home-manager.users.admin.assertions':

       … while evaluating definitions from `/nix/store/3ipnrh8kacbhawf90zsi5y0i8kpv7b1b-source/modules/systemd.nix':

       … while evaluating the option `home-manager.users.admin.home.username':

       … while evaluating definitions from `/nix/store/3ipnrh8kacbhawf90zsi5y0i8kpv7b1b-source/nixos/common.nix':

       … while evaluating the option `users.users':

       … while evaluating definitions from `/nix/store/f9wi26djyrxzcg79l4r593vp45yfljhi-source/modules/bitcoind.nix':

       … while evaluating the option `services.bitcoind.enable':

       (stack trace truncated; use '--show-trace' to show the full, detailed trace)

       error: The option `services.bitcoind.enable' has conflicting definition values:
       - In `/nix/store/hx4lg3k7qz0nc4bar9qyxv0a05c1jaj7-source/src/btc.nix': false
       - In `/nix/store/f9wi26djyrxzcg79l4r593vp45yfljhi-source/modules/lnd.nix': true
       Use `lib.mkForce value` or `lib.mkDefault value` to change the priority on any of these definitions.
  --- Process finished with status: exit status: 1 ---
  Error applying changes: Unknown error
  ├╴at cli/src/commands/apply.rs:12:10
  │
  ╰─▶ Unable to apply changes
    ├╴at /build/packages/nixblitzlib/src/utils.rs:254:33
    ╰╴Command exited with non-zero status.
  ```

- mounting the webapp to a subpath like `/app` or `/foo/bar` doesn't work
