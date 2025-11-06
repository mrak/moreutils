setup() {
  load 'test_helper/bats-support/load'
  load 'test_helper/bats-assert/load'
  DIR="$( cd "$( dirname "$BATS_TEST_FILENAME" )" >/dev/null 2>&1 && pwd )"
  PATH="$( realpath "$DIR/../target/debug"):$PATH"
  TEST_IN=$(mktemp)
  TEST_OUT=$(mktemp -u)
}

teardown() {
  rm -f "$TEST_IN" "$TEST_OUT"
}

@test "Can absorb stdin into file without clobbering it" {
  echo test > "$TEST_IN"
  sed -e 's|e|a|' "$TEST_IN"| sponge "$TEST_IN"
  assert [ "$(cat "$TEST_IN")" = tast ]
}

@test "Can absorb stdin into file while appending it" {
  printf %s test > "$TEST_IN"
  sed -e 's|e|a|' "$TEST_IN"| sponge -a "$TEST_IN"
  assert [ "$(cat "$TEST_IN")" = "testtast" ]

  sed -e 's|[ea]|i|g' "$TEST_IN"| sponge "$TEST_IN" -a
  assert [ "$(cat "$TEST_IN")" = "testtasttisttist" ]
}

@test "Can absorb stdin into new file" {
  printf %s test > "$TEST_IN"
  sed -e 's|e|a|' "$TEST_IN"| sponge "$TEST_OUT"
  assert [ "$(cat "$TEST_OUT")" = "tast" ]
}

@test "Can absorb stdin into new appended file" {
  printf %s test > "$TEST_IN"
  sed -e 's|e|a|' "$TEST_IN"| sponge -a "$TEST_OUT"
  assert [ "$(cat "$TEST_OUT")" = "tast" ]
}

@test "Can absorb stdin to stdout" {
  printf %s test > "$TEST_IN"
  run sponge < "$TEST_IN"
  assert_output "test"
}

@test "usage: unknown arg stderr" {
  printf %s test > "$TEST_IN"
  run sponge --unknown "$TEST_IN" < "$TEST_IN"
  assert [ "$status" -eq 1 ]
  assert_output "invalid option '--unknown'
Usage: sponge [-a] FILE"
}
