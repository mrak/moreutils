.PHONY: install symlinks uninstall src/errno/errno_generated.rs
IMPLEMENTED := sponge vipe ts vidir ifne pee errno combine
SYMLINKS := $(addprefix ${HOME}/.local/bin/, $(IMPLEMENTED))

install: symlinks src/errno/errno_generated.rs
	cargo install --path . --root ${HOME}/.local

uninstall:
	rm $(SYMLINKS)

symlinks: $(SYMLINKS)

$(SYMLINKS):
	ln -sf moreutils $@

src/errno/errno_generated.rs:
	echo "#include <errno.h>" | $(CC) -E -dD -x c - | awk -f "scripts/errno_generated.awk" > $@
