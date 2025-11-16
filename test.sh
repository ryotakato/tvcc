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
try 21 "5+20-4"
try 41 " 12 + 34 - 5 "
try 47 "5+6*7"
try 15 "5*(9-6)"
try 4 "(3+5)/2"
try 10 "-10+20"

echo OK
