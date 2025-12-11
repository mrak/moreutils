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

@test "Prepends the incremental timestamp in %H:%M:%S format" {
  (
    echo "one"
    sleep 1
    echo "two"
    sleep 1
    echo "three"
  ) | ts -i > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "00:00:00 one
00:00:01 two
00:00:01 three"
}

@test "Prepends the incremental timestamp in custom format" {
  (
    echo "one"
    sleep 1
    echo "two"
    sleep 1
    echo "three"
  ) | ts -i "%S" > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "00 one
01 two
01 three"
}

@test "Prepends the since-start timestamp in %H:%M:%S format" {
  (
    echo "one"
    sleep 1
    echo "two"
    sleep 1
    echo "three"
  ) | ts -s > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "00:00:00 one
00:00:01 two
00:00:02 three"
}

@test "Prepends the since-start timestamp in custom format" {
  (
    echo "one"
    sleep 1
    echo "two"
    sleep 1
    echo "three"
  ) | ts -s "%S" > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "00 one
01 two
02 three"
}

@test "Relative time with RFC3339" {
  (
    echo "$(date -v-2S +%Y-%m-%dT%H:%M:%S%z|sed -E 's/([+-][0-9]{2})([0-9]{2})$/\1:\2/') one"
    echo "$(date -v-1S +%Y-%m-%dt%H:%M:%S%z|sed -E 's/([+-][0-9]{2})([0-9]{2})$/\1:\2/') two"
    echo "$(date +'%Y-%m-%d %H:%M:%S%z'|sed -E 's/([+-][0-9]{2})([0-9]{2})$/\1:\2/') three"
  ) > "$TEST_IN"
  ts -r < "$TEST_IN" > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "2s ago one
1s ago two
just now three"
}

@test "Relative time with RFC3339 but : absent from offset" {
  (
    echo "$(date -v-2S +%Y-%m-%dT%H:%M:%S%z) one"
    echo "$(date -v-1S +%Y-%m-%dt%H:%M:%S%z) two"
    echo "$(date +'%Y-%m-%d %H:%M:%S%z') three"
  ) > "$TEST_IN"
  ts -r < "$TEST_IN" > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "2s ago one
1s ago two
just now three"
}

@test "Relative time with RFC2822" {
  (
    echo "$(date -v-2S +"%a, %d %b %Y %H:%M:%S %z") one"
    echo "$(date -v-1S +"%a, %d %b %Y %H:%M:%S %Z") two"
    echo "$(date +"%d %b %Y %H:%M:%S %Z") three"
  ) > "$TEST_IN"
  ts -r < "$TEST_IN" > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "2s ago one
1s ago two
just now three"
}

@test "Relative time with unix timestamps (seconds)" {
  (
    echo "$(date -v-2S +"%s") one"
    echo "$(date -v-1S +"%s") two"
    echo "$(date +"%s") three"
  ) > "$TEST_IN"
  ts -r < "$TEST_IN" > "$TEST_OUT"
  assert_equal "$(cat "$TEST_OUT")" "2s ago one
1s ago two
just now three"
}

# TODO lastlog
