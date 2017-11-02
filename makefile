devel: holysee-devel holygram-devel holyirc-devel holyrpc-devel

holysee-devel:
	cd holysee; cargo build

holygram-devel:
	cd holygram; cargo build

holyirc-devel:
	cd holyirc; cargo build

holyrpc-devel:
	cd holyrpc; cargo build

release: holysee holygram holyirc holyrpc

holysee:
	cd holysee; cargo build --release

holygram:
	cd holygram; cargo build --release

holyirc:
	cd holyirc; cargo build --release

holyrpc:
	cd holyrpc; cargo build --release

all: devel
