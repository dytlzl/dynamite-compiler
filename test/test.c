int n;

int assert(int expected, int actual) {
    n = n+1;
    printf("%d: ", n);
    if (expected == actual) {
        printf("%d, OK\n", expected);
    } else {
        printf("%ld expected, but got %ld\n", expected, actual);
        exit(1);
    }
    return 0;
}
// single line comment
int fib_memo[47];
char a;//, aa;
/*
 * multiline comment
 */


int fib(int i) {
    if (i == 0) return 0;
    if (i == 1) return 1;
    if (fib_memo[i] != 0) {
        return fib_memo[i];
    }
    return fib_memo[i] = fib(i-1) + fib(i-2);
}

int main() {
    n = 0;
    assert(0, 0);
    assert(42, 42);
    assert(-7, -7);
    assert(28, 3 * (29 % (13-2) + 3) - 2);
    assert(10, -1* 4+2*+7);
    assert(1, 5-3==2);
    assert(1, 123<31*4);
    assert(1, 124<=31*4);
    assert(0, 124 > 31*4);
    assert(1, 124>=31*4);
    a = -3;
    assert(1836311903, fib(46));
    int b, c = 4, *d, e[3];
    e[2] = 5; b = 3; d = &e[2];
    assert(60, b*c**d);
    for (int i = 0;;) {
        break;
    }
    char ch[3];
    int f;
    f = 4;
    printf("%lu\n", &f);
    return 0;
}