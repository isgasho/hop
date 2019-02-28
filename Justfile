# wengwengweng

run:
	cargo run --release

loc:
	tokei

checkdep:
	cargo outdated --depth 1

test-windows :
	cargo build --release --target x86_64-pc-windows-gnu


