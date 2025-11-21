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

try 0 "0;"
try 42 "42;"
try 21 "5+20-4;"
try 41 " 12 + 34 - 5 ;"
try 47 "5+6*7;"
try 15 "5*(9-6);"
try 4 "(3+5)/2;"
try 10 "-10+20;"
try 10 "- -10;"
try 10 "- - +10;"

try 0 "0==1;"
try 1 "42==42;"
try 1 "0!=1;"
try 0 "42!=42;"

try 1 "0<1;"
try 0 "1<1;"
try 0 "2<1;"
try 1 "0<=1;"
try 1 "1<=1;"
try 0 "2<=1;"

try 1 "1>0;"
try 0 "1>1;"
try 0 "1>2;"
try 1 "1>=0;"
try 1 "1>=1;"
try 0 "1>=2;"

try 1 "a=1;a;"
try 1 "b=1;b;"
try 1 "z=1;z;"
try 3 "r=1;t=2;o=r+t;o;"
try 9 "t=4; t = t + 5;"

try 3 "1; 2; 3;"

try 3 "a=3; a;"
try 8 "a=3; z=5; a+z;"
try 6 "a=b=3; a+b;"

try 3 'foo=3; foo;'
try 8 'foo123=3; bar=5; foo123+bar;'

try 1 'return 1; 2; 3;'
try 2 '1; return 2; 3;'
try 3 '1; 2; return 3;'

echo OK
