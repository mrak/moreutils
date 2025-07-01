.PHONY: install symlinks uninstall src/errno/errno_generated.rs
IMPLEMENTED := sponge vipe ts vidir ifne pee errno
SYMLINKS := $(addprefix ${HOME}/.local/bin/, $(IMPLEMENTED))

install: symlinks src/errno/errno_generated.rs
	cargo install --path . --root ${HOME}/.local

uninstall:
	rm $(SYMLINKS)

symlinks: $(SYMLINKS)

$(SYMLINKS):
	ln -sf moreutils $@

src/errno/errno_generated.rs:
	echo "#include <errno.h>" > errno.c
	echo >  src/errno/errno_generated.rs "use std::borrow::Cow;"
	echo >> src/errno/errno_generated.rs
	echo >> src/errno/errno_generated.rs "pub struct Errno {"
	echo >> src/errno/errno_generated.rs "    pub name: Cow<'static, str>,"
	echo >> src/errno/errno_generated.rs "    pub id: i32,"
	echo >> src/errno/errno_generated.rs "}"
	echo >> src/errno/errno_generated.rs
	echo >> src/errno/errno_generated.rs "pub const ERRNOS: &[Errno] = &["
	$(CC) -E -dD errno.c | awk >> src/errno/errno_generated.rs '/^ *#define E/ && $$3 ~ /[[:digit:]]{1,}/ {errnos[$$2] = $$3;next} /^ *#define E/ {errnos[$$2] = errnos[$$3]} END { for (errno in errnos) printf("    Errno { name: Cow::Borrowed(\"%s\"), id: %d },\n", errno, errnos[errno])}'
	echo >> src/errno/errno_generated.rs "];"
	rm errno.c
