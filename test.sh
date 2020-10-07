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

assert 42 'main() { return 42; }'
assert 28 'main() { return 3 * (29 % (13-2) + 3) - 2; }'
assert 10 'main() { return -1* 4+2*+7; }';
assert 1 'main() { return 5-3==2; }'
assert 1 'main() { return 123<31*4; }'
assert 1 'main() { return 124<=31*4; }'
assert 0 'main() { return 124>31*4; }'
assert 1 'main() { return 124>=31*4; }'
assert 12 'main() { a=3; b=4; return a*b; }'
assert 1 'main() { a=3; b=4; return a*a*b==36; }'
assert 31 'main() { ice=3; cream=7; return ice*cream+ice+cream; }'
assert 27 'main() { a=3; b = c = a; return a*b*c; }'
assert 66 'main() { a=47; b=19; return a+b; return a%b; }'
assert 5 'main() { if (3*6>15) return 5; else return 10; }'
assert 7 'main() { x = 5; if (x > 4) x = x+5; return x-3; }'
assert 12 'main() { a = 3; while (a*a < 100) a=a+3; a; }'
assert 45 'main() { a = 0; for(i = 0; i < 10; i = i+1) a = a+i; return a; }'
assert 12 'main() { a = 0; for (;; a = a+3) if (a >= 10) break; return a; }'
assert 50 'main() { a = 0; for (i = 0; i < 10; i = i+1) { j = 0; while (j < 5) { a = a+1; j = j+1; } } return a; }'
assert 0 'main() { for (i = 0; i < 1;) { break; } return i; }'
assert 10 'main() { a = 0; j = 0; while (1) { if (j>=5) break; a = a+j; j = j+1; } return a; }'
assert 50 'main() { a = 0; for (i = 0; i < 10; i = i+1) { j = 0; while (1) { if (j>=5) break; a = a+1; j = j+1; } } return a; }'
assert 104 'main() { return add6(Add(3, 8), 2, 3, 4, 5, 6); } Add(a, b) { return a*2+b; } add6(a, b, c, d, e, f) { return a + b*2 + c*3 + d*4 + e*5 + f*6; }'

echo OK
