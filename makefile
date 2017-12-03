all: devel
release: holysee
devel: holysee-devel

holysee-devel: run
	cd holysee; cargo build

holysee: run
	cd holysee; cargo build --release

install: run holysee
	cd holysee; cargo install

test: run holysee-devel
	cd holysee; cargo cov test; cargo cov report --open

.PHONY: run
