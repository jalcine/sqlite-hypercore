#!/usr/bin/env sh

cargo test --color never -q --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]"
exit 0
