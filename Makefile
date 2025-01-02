test:
	find . -name 'Cargo.toml' -execdir cargo test \;

lint:
	find . -name 'Cargo.toml' -execdir cargo clippy \;
