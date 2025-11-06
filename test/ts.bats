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

@test "Prepends the current date in the default format" {
  (
    echo "one"
    echo "two"
    echo "three"
  ) | ts > "$TEST_OUT"
  while read -r line; do
    run echo "$line"
    assert_output --regexp '^[A-Za-z]{3} [0-9]{1,2} [0-9]{2}:[0-9]{2}:[0-9]{2} (one|two|three)'
  done < "$TEST_OUT"
}

@test "Prepends the current date with a custom format" {
  (
    echo "one"
    echo "two"
    echo "three"
  ) | ts 'Time %H hours and %M minutes' > "$TEST_OUT"
  while read -r line; do
    run echo "$line"
    assert_output --regexp '^Time [0-9]{2} hours and [0-9]{2} minutes (one|two|three)'
  done < "$TEST_OUT"
}
