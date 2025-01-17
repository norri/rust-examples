test:
	find . -name 'Cargo.toml' -execdir cargo nextest run \;

lint:
	find . -name 'Cargo.toml' -execdir bash -c 'cargo fmt --check && cargo clippy' \;
