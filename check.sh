#!/bin/bash

cargo clippy --all-targets --all-features --target x86_64-unknown-linux-gnu -- -W clippy::pedantic -Dwarnings
