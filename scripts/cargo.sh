#!/bin/sh

export CARGO_HOME=$1/target/cargo-home

if [[ $DEBUG = true ]]
then
    echo "DEBUG MODE"
    cargo build && cp $1/target/debug/visol-rs $2
else
    echo "RELEASE MODE"
    cargo build --release && cp $1/target/release/visol-rs $2
fi
