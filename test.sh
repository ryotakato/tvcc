#!/bin/bash

#CMD=./target/debug/tvcc
CMD=./target/x86_64-unknown-linux-musl/debug/tvcc
TARGET=./target/tmp

mkdir -p "${TARGET}"

try() {
  expected="$1"
  input="$2"

  ${CMD} "$input" > "${TARGET}/tmp.s"
  gcc -o "${TARGET}/tmp" "${TARGET}/tmp.s"
  "${TARGET}/tmp"
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

try 0 0
try 42 42

echo OK
