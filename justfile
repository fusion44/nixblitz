rust_src := "./packages"

set positional-arguments

sync-to-blitz:
  rsync -avPzu --delete-during --progress src/ admin@192.168.8.242:/home/admin/dev/sys
  rsync -avPzu --delete-during --progress ../api/nixosify/ admin@192.168.8.242:/home/admin/dev/api
  rsync -avPzu --delete-during --progress ../web/nixosify/ admin@192.168.8.242:/home/admin/dev/web
  rsync -avPzu --delete-during --progress --exclude="history.txt" src/configs/nushell/ admin@192.168.8.242:/home/admin/.config/nushell

# format all Nix files
format:
  alejandra	.
  cd {{rust_src}} && cargo fmt

lint: 
  cd {{rust_src}} && cargo check

# run the CLI, any args are passed to the CLI unaltered 
run-cli *args='':
  cd {{rust_src}} && cargo run $@

# builds the current config as a qemu vm
vm-build:
	cd src && export QEMU_NET_OPTS="hostfwd=tcp::2221-:22,hostfwd=tcp::8080-:80" && nixos-rebuild build-vm --flake .#devsys

# runs the current qemu vm
vm-run:
	./src/result/bin/run-tbnix_vm-vm

# ssh into the currently running qemu vm
vm-ssh:
	ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no admin@localhost -p 2221

# resets the vm by deleting the tbnix_vm.qcow2 file
vm-reset:
	rm -i tbnix_vm.qcow2


