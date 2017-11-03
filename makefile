all: devel
release: holysee 
devel: holysee-devel 

holysee-devel: run
	cd holysee; cargo build

holysee: run
	cd holysee; cargo build --release

.PHONY: run
