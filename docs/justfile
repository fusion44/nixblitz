# Prints this help text
help:
	just --list

# Serve the documentation locally
serve:
   npm start

# Build the documentation files
build:
   npm run build

# Update all documentation
update-all:
	npm update
	yarn upgrade
	nix flake update

# Format the markdown files
format:
  prettier -w .
