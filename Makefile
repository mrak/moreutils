.PHONY = install

install: symlinks
	cargo install --path . --root ${HOME}/.local

symlinks:
	ln -sf moarutils ${HOME}/.local/bin/sponge
	ln -sf moarutils ${HOME}/.local/bin/vipe
