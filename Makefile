test:
	find . -name 'Cargo.toml' -execdir cargo nextest run \;

lint:
	find . -name 'Cargo.toml' -execdir cargo clippy \;
