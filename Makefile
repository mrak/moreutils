.PHONY: install symlinks uninstall src/errno/errno_generated.rs test
IMPLEMENTED := chronic combine errno ifdata ifne isutf8 mispipe parallel pee sponge ts vidir vipe zrun
SYMLINKS := $(addprefix ${HOME}/.local/bin/, $(IMPLEMENTED))
DEV_SYMLINKS := $(addprefix target/debug/, $(IMPLEMENTED))

install: symlinks src/errno/errno_generated.rs
	cargo install --path . --root ${HOME}/.local

uninstall:
	rm $(SYMLINKS)

symlinks: $(SYMLINKS)

$(SYMLINKS):
	ln -sf moreutils $@

$(DEV_SYMLINKS):
	ln -sf moreutils $@

src/errno/errno_generated.rs:
	echo "#include <errno.h>" | $(CC) -E -dD -x c - | awk -f "scripts/errno_generated.awk" > $@

build:
	cargo build

test: build $(DEV_SYMLINKS)
	./test/bats/bin/bats test/*.bats
