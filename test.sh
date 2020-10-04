#!/bin/bash
assert() {
  expected="$1"
  input="$2"
  ./target/debug/dynamite_compiler "$input" > asm/main.s
  cc -o ./bin/main ./asm/main.s
  ./bin/main
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}
if [ ! -d ./asm ]; then
  mkdir ./asm
fi
if [ ! -d ./bin ]; then
  mkdir ./bin
fi
<< CMT
CMT

assert 42 '42;'
assert 28 '3 * (29 % (13-2) + 3) - 2;'
assert 10 '-1* 4+2*+7;'; # unary
assert 1 '5-3==2;'
assert 1 '123<31*4;'
assert 1 '124<=31*4;'
assert 0 '124>31*4;'
assert 1 '124>=31*4;'
assert 12 'a=3; b=4; a*b;'
assert 1 'a=3; b=4; a*a*b==36;'
assert 31 'ice=3; cream=7; ice*cream+ice+cream;'
assert 66 'a=47; b=19; return a+b; a%b;'
assert 5 'if (3*6>15) 5; else 10;'
assert 7 'x = 5; if (x > 4) x = x+5; x-3;'


echo OK
