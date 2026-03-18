#!/bin/env bash

set -e

APP="${1:-backend}"
echo $APP

if [ "$APP" != "backend" ] && [ "$APP" != "stream-handler" ]; then
    echo >&2 ERROR: you must supply an argument, either 'backend' or 'stream-handler'
    exit 1
fi

pushd famtrac-"$APP"

cargo build --release
test -f target/release/famtrac-"$APP"
cp target/release/famtrac-"$APP" bootstrap
zip build_famtrac_"${APP/-/_}"_function.zip bootstrap

popd