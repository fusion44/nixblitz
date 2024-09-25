rust_src := "./packages"

set positional-arguments

# Lists available commands
default:
	just --list

# clean the workspace
clean:
  alejandra	.
  cd {{rust_src}} && cargo clean 

# format all Nix files
format:
  alejandra	.
  cd {{rust_src}} && cargo fmt

# Run all lints and checks
lint: 
  cd {{rust_src}} && cargo check && cargo deny check

# runs all tests
test:
	cd {{rust_src}} && cargo test

# run the CLI, any args are passed to the CLI unaltered 
run-cli *args='':
  cd {{rust_src}} && cargo run $@

