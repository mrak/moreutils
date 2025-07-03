BEGIN {
  print "use std::borrow::Cow;"
  print ""
  print "pub struct Errno {"
  print "    pub name: Cow<'static, str>,"
  print "    pub id: i32,"
  print "}"
  print ""
  print "pub const ERRNOS: &[Errno] = &["
}

/^ *#define  *E/ {
  if ($3 ~ /[[:digit:]]{1,}/) errnos[$2] = $3
  else                        errnos[$2] = errnos[$3]
}

END {
  for (errno in errnos) {
    printf("    Errno { name: Cow::Borrowed(\"%s\"), id: %d },\n", errno, errnos[errno])
  }
  print "];"
}
