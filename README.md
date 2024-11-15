# nixblitz

### FAQ

#### Where to find logs in TUI mode?

In TUI mode, no error messages are printed to stdout. This would would interfere
with the UI drawing process. Instead, it the log will be written
to a file called `nixblitz.log` in the working directory.
This behavior is **off** by default.

The CLI uses [cli-log](https://crates.io/crates/cli-log) to log
to the file. To enable logging, the `NIXBLITZ_LOG` env variable
must to be set.

A convenient way to set the env variable is to start the app as

- bash `NIXBLITZ_LOG=debug nixblitz tui`
- nushell `$env.NIXBLITZ_LOG = debug; nixblitz tui`

or, during development,

- bash `NIXBLITZ_LOG=debug cargo run tui`
- nushell `$env.NIXBLITZ_LOG = debug; cargo run tui`
