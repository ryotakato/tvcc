#!/bin/bash

#CMD=./target/debug/tvcc
CMD=./target/x86_64-unknown-linux-musl/debug/tvcc
TARGET=./target/tmp

mkdir -p "${TARGET}"

cat <<EOF | gcc -xc -c -o "${TARGET}/tmp2.o" -
int ret3() { return 3; }
int ret5() { return 5; }
int add(int x, int y) { return x+y; }
int sub(int x, int y) { return x-y; }
int add6(int a, int b, int c, int d, int e, int f) {
  return a+b+c+d+e+f;
}
EOF

try() {
  expected="$1"
  input="$2"

  ${CMD} "$input" > "${TARGET}/tmp.s"
  gcc -o "${TARGET}/tmp" "${TARGET}/tmp.s" "${TARGET}/tmp2.o"
  "${TARGET}/tmp"
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

try 0 "{return 0;}"
try 42 "{return 42;}"
try 21 "{return 5+20-4;}"
try 41 "{return 12 + 34 - 5 ;}"
try 47 "{return 5+6*7;}"
try 15 "{return 5*(9-6);}"
try 4 "{return (3+5)/2;}"
try 10 "{return -10+20;}"
try 10 "{return - -10;}"
try 10 "{return - - +10;}"

try 0 "{return 0==1;}"
try 1 "{return 42==42;}"
try 1 "{return 0!=1;}"
try 0 "{return 42!=42;}"

try 1 "{return 0<1;}"
try 0 "{return 1<1;}"
try 0 "{return 2<1;}"
try 1 "{return 0<=1;}"
try 1 "{return 1<=1;}"
try 0 "{return 2<=1;}"

try 1 "{return 1>0;}"
try 0 "{return 1>1;}"
try 0 "{return 1>2;}"
try 1 "{return 1>=0;}"
try 1 "{return 1>=1;}"
try 0 "{return 1>=2;}"

try 1 " { a=1; return a;}"
try 1 " { b=1;return b;}"
try 1 " { z=1;return z;}"
try 3 " { r=1;t=2;o=r+t;return o;}"
try 9 " { t=4; t = t + 5;return t;}"

try 3 "{1; 2; return 3;}"

try 3 "{ a=3; return a; }"
try 8 "{ a=3; z=5; return a+z; } "
try 6 "{ a=b=3; return a+b; } "

try 3 ' { foo=3; return foo; }'
try 8 ' { foo123=3; bar=5; return foo123+bar; }'

try 1 ' { return 1; 2; 3; } '
try 2 ' { 1; return 2; 3; } '
try 3 ' { 1; 2; return 3; } '

try 3 '{ {1; {2;} return 3;} }'

try 5 '{ ;;; return 5;}'

try 3 '{ if (0) return 2; return 3; }'
try 3 '{ if (1-1) return 2; return 3; }'
try 2 '{ if (1) return 2; return 3; }'
try 2 '{ if (2-1) return 2; return 3; }'
try 4 '{ if (0) return 3; else return 4; }'
try 3 '{ if (1) return 3; else return 4; }'

try 55 '{ i=0; j=0; for (i=0; i<=10; i=i+1) j=i+j; return j; }'
try 3 ' { for (;;) return 3; return 5; }'
try 10 '{ i=0; while(i<10) i=i+1; return i; }'

try 3 '{ return ret3(); }'
try 5 '{ return ret5(); }'
try 8 '{ return add(3, 5); }'
try 2 '{ return sub(5, 3); }'
try 21 '{ return add6(1,2,3,4,5,6); }'
try 66 '{ return add6(1,2,add6(3,4,5,6,7,8),9,10,11); }'
try 136 '{ return add6(1,2,add6(3,add6(4,5,6,7,8,9),10,11,12,13),14,15,16); }'
try 136 '{ return add6(1,2,add6(3,add6(4,5,6,7,8,9),10,11,12,13),14,15,16,17); }' # ignore 7th arity

echo OK
