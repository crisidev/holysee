all: devel
release: holysee holygram holyirc holyrpc
devel: holysee-devel holygram-devel holyirc-devel holyrpc-devel

holysee-devel: run
	cd holysee; cargo build

holygram-devel: run
	cd holygram; cargo build

holyirc-devel: run
	cd holyirc; cargo build

holyrpc-devel: run
	cd holyrpc; cargo build

holysee: run
	cd holysee; cargo build --release

holygram: run
	cd holygram; cargo build --release

holyirc: run
	cd holyirc; cargo build --release

holyrpc: run
	cd holyrpc; cargo build --release

.PHONY: run
