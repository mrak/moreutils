.PHONY: install symlinks uninstall
IMPLEMENTED := sponge vipe ts vidir
SYMLINKS := $(addprefix ${HOME}/.local/bin/, $(IMPLEMENTED))

install: symlinks
	cargo install --path . --root ${HOME}/.local

uninstall:
	rm $(SYMLINKS)

symlinks: $(SYMLINKS)

$(SYMLINKS):
	ln -sf moreutils $@
