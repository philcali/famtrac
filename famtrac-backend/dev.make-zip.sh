#!/bin/env bash

set -e

cargo build --release
test -f target/release/famtrac-backend
cp target/release/famtrac-backend bootstrap
zip build_famtrac_function.zip bootstrap