set shell := ["nu", "-c"]
rust_src := "./packages"

set positional-arguments

# Lists available commands
default:
	just --list

# clean the workspace
clean:
  alejandra	.
  cd {{rust_src}}; cargo clean

# format all Nix files
format:
  alejandra	.
  cd {{rust_src}}; cargo fmt

# Run lints and checks; Pass -f to apply auto fix where possible
lint fix="":
  #!/usr/bin/env nu
  if ("{{fix}}" == "") {
    typos
    cd {{rust_src}}
    cargo clippy --workspace -- --no-deps
    cargo fmt --all -- --check
  } else if ("{{fix}}" == "-f") {
    typos -w
    cd {{rust_src}}
    cargo clippy --fix --allow-dirty --allow-staged --workspace -- --no-deps
    cargo fmt --all
  } else {
    print "Unknown argument '{{fix}}'. Pass '-f' to auto fix or nothing to dry run."
  }

# runs all tests
test:
	cd {{rust_src}}; cargo test

# run the CLI with debug log enabled, any args are passed to the CLI unaltered
run-cli *args='':
  cd {{rust_src}}; $env.NIXBLITZ_LOG = "trace"; cargo run {{args}}

# shorthand for rsync this source directory to a remote node.
rsync target:
  #!/usr/bin/env nu
  if not ('.remotes.json' | path exists) {
    print "Config file '.remotes.json' not found."
    print "Find an example template '.remotes.json.sample'"
    exit 1
  }

  let data = open .remotes.json
  if ($data | columns | "all" in $in ) {
    print "The keyword 'all' is reserved to rsync to all the remotes declared in the .remotes.json file"
    exit 1
  }

  if ("{{target}}" == "all") {
    $data | transpose key value | each { |remote|
      print $"Syncing ($remote.key)"
      let data2 = $remote.value
      let cmd = $data2.user + "@" + $data2.host + ":" + $data2.path
      rsync -rvz --exclude .git --exclude docs/ --exclude packages/target/ . $cmd
    }
    exit 0
  } else {
    print $"Syncing {{target}}"
    let $data = $data | get {{target}}
    let cmd = $data.user + "@" + $data.host + ":" + $data.path
    rsync -rvz --exclude .git --exclude docs/ --exclude packages/target/ . $cmd
  }
