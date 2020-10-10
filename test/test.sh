#!/bin/bash

options=$1

assert() {
  expected="$1"
  input="$2"
  ./target/debug/dynamite_compiler "$input" > ./temp/main.s
  if [ $? != 0 ]; then
    exit 1
  fi
  cc $options -o ./temp/main ./temp/main.s
  if [ $? != 0 ]; then
    exit 1
  fi
  ./temp/main
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert_stdout() {
  expected="$1"
  input="$2"
  ./target/debug/dynamite_compiler "$input" > ./temp/main.s
  if [ $? != 0 ]; then
    exit 1
  fi
  cc $options -o ./temp/main ./temp/main.s
  if [ $? != 0 ]; then
    exit 1
  fi
  actual=`./temp/main`

  if [ "$actual" = "$expected" ]; then
    echo "( $input ) => ( $actual )"
  else
    echo "[ $input ) => ( $expected ) expected, but got ( $actual )"
    exit 1
  fi
}

if [ ! -d ./temp ]; then
  mkdir ./temp
fi
<< CMT
CMT

assert 0 './test/test.c'
assert_stdout "value = 777" 'int value; int main() { value = 777; printf("value = %d\n", value); return 0;}'
assert_stdout "value = 755" 'int value; int main() { char* fmt = "value = %d\n"; value = 755; printf(fmt, value); return 0;}'
assert_stdout "value = 222" 'char* fmt; int main() { fmt = "value = %d\n"; printf(fmt, 222); return 0;}'

echo OK
