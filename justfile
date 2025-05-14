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

update-flakes mode="nixblitz":
  #!/usr/bin/env nu
  fd flake.lock | lines | path dirname | each { |d|
    cd $d
    print $"Updating flakes in ($d)"
    let cmd = if ("{{mode}}" == "full") {
      nix flake update
    } else if ("{{mode}}" == "nixblitz") {
      nix flake update nixblitz
    } else {
      print "Unknown mode '{{mode}}'. Valid modes are 'full' and 'nixblitz'."
    }
  }

# inside the test vm: sync from shared folder to dev
sync-src-temp:
  rsync -av --exclude='target/' --exclude='.git' --exclude='result' /mnt/shared/ /home/nixos/dev

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

# runs all tests; Pass --trace (-t) to enable Rust tracing
test trace="":
  #!/usr/bin/env nu
  if ("{{trace}}" == "-t" or "{{trace}}" == "--trace") {
    cd {{rust_src}}
    $env.RUST_BACKTRACE = 1
    cargo test
  } else if ("{{trace}}" == "") {
    cd {{rust_src}}
    cargo test
  } else {
    print "Unknown argument '{{trace}}'. Pass '-t' to enable Rust tracing or nothing to run without it."
  }

# run the CLI with debug log enabled, any args are passed to the CLI unaltered
run-cli *args='':
  cd {{rust_src}}; $env.RUST_BACKTRACE = 1; $env.NIXBLITZ_LOG = "trace"; cargo run {{args}}

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

build-installer verbosity="normal":
  #!/usr/bin/env nu
  let has_untracked_files = (
    try {
        git status --porcelain=v1 | lines | where { |it| $it | str starts-with "?? packages" } | is-not-empty
    } catch {
        # If git status fails, assume there might be issues (treat as "changes found")
        true
    }
  )

  if $has_untracked_files {
    # Nix builds the CLI and runs tests, but skips untracked git files,
    # causing potential build failures if templates are missing.
    print -e $"\e[33m\u{26A0} Warning: You have unstaged changes or untracked files. Build and tests may fail.\e[0m"
  }

  if ("{{verbosity}}" == "verbose") {
    print "Building installer ISO image with verbosity level '{{verbosity}}'"
    (
      nix build
        -L
        --no-update-lock-file
        './installer#nixosConfigurations.nixblitzx86installer.config.system.build.isoImage'
    )
  } else {
    print "Building installer ISO image with verbosity level '{{verbosity}}'"
    (
      nix build
        --no-update-lock-file
        './installer#nixosConfigurations.nixblitzx86installer.config.system.build.isoImage'
    )
  }



run-installer-vm target='default':
  #!/usr/bin/env nu
  if not ('result/iso' | path exists) {
    print "Iso file not found. Run 'just build-installer' first."
    exit 1
  }

  let iso_name = ls result/iso | first | get name

  if not ('nixblitz-disk.qcow2' | path exists) {
    try {
      print "Enter the size of the image in GB:"
      let res = input | into int
      print $"Creating image file 'nixblitz-disk.qcow2' with ($res)G."
      qemu-img create -f qcow2 nixblitz-disk.qcow2 $'($res)G'
    } catch {
      print "Input must be a number."
      exit 1
    }
  }

  if ("{{target}}" == "default" or "{{target}}" == "single") {
    print "Running installer VM with a single virtio disk"
    (qemu-system-x86_64 -enable-kvm -m 16384 -smp 4
      -netdev user,id=mynet0,hostfwd=tcp::10022-:22
      -device virtio-net-pci,netdev=mynet0
      -drive file=nixblitz-disk.qcow2,if=none,id=virtio0,format=qcow2
      -device virtio-blk-pci,drive=virtio0
      -cdrom $iso_name)
  } else if ("{{target}}" == "dual") {
    print "Running installer with a local disk and usb attached disk"
  } else {
    print "Unknown target '{{target}}'. Valid targets are 'default', 'single' and 'dual'."
  }

run-installed-vm:
  #!/usr/bin/env nu
  nu installer/tools/run_vm.nu
