#!/bin/env bash

set -e

APP="${1:-backend}"
echo $APP

if [ "$APP" != "backend" ] && [ "$APP" != "stream-handler" ] && [ "$APP" != "authorizer" ]; then
    echo >&2 ERROR: you must supply an argument, either 'backend', 'stream-handler', or 'authorizer'
    exit 1
fi

pushd famtrac-"$APP"

cargo build --release
test -f target/release/famtrac-"$APP"
cp target/release/famtrac-"$APP" bootstrap
zip build_famtrac_"${APP/-/_}"_function.zip bootstrap

popd