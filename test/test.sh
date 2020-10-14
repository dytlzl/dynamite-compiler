#!/bin/bash

options=$1

assert() {
  expected="$1"
  input="$2"
  assembly=`./target/debug/dynamite_compiler "$input"`
  if [ $? != 0 ]; then
    exit 1
  fi
  echo "$assembly" | cc $options -x assembler -o ./temp/main -
  if [ $? != 0 ]; then
    exit 1
  fi
  if [ "$3" = "stdout" ]; then
    actual=`./temp/main`
  else
    ./temp/main
    actual="$?"
  fi
  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

if [ ! -d ./temp ]; then
  mkdir ./temp
fi

assert 0 './test/expr.c'
assert 0 './test/functions.c'
assert "a = 777, b = 755, c = 222" './test/string.c' stdout

echo OK
