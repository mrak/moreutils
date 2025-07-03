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
	echo > errno.c "#include <errno.h>"
	echo >  $@ "use std::borrow::Cow;"
	echo >> $@
	echo >> $@ "pub struct Errno {"
	echo >> $@ "    pub name: Cow<'static, str>,"
	echo >> $@ "    pub id: i32,"
	echo >> $@ "}"
	echo >> $@
	echo >> $@ "pub const ERRNOS: &[Errno] = &["
	$(CC) -E -dD errno.c | awk >> $@ '/^ *#define E/ && $$3 ~ /[[:digit:]]{1,}/ {errnos[$$2] = $$3;next} /^ *#define E/ {errnos[$$2] = errnos[$$3]} END { for (errno in errnos) printf("    Errno { name: Cow::Borrowed(\"%s\"), id: %d },\n", errno, errnos[errno])}'
	echo >> $@ "];"
	rm errno.c
