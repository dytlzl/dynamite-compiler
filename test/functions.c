int n = 0;

int assert(int expected, int actual) {
    n = n + 1;
    printf("%d: ", n);
    if (expected == actual) {
        printf("%d, OK\n", expected);
    } else {
        printf("%d expected, but got %d\n", expected, actual);
        exit(1);
    }
    return 0;
}

// single line comment
int fib_memo[47];
char a, aa;

/*
 * multiline comment
 */

int add6(int a, int b, int c, int d, int e, int f) {
    return a + b * 2 + c * 3 + d * 4 + e * 5 + f * 6;
}

int Add(int a, int b) { return a * 2 + b; }

int fib(int i) {
    if (i == 0) return 0;
    if (i == 1) return 1;
    if (fib_memo[i] != 0) {
        return fib_memo[i];
    }
    return fib_memo[i] = fib(i - 1) + fib(i - 2);
}

int test01() {
    int a = 3;
    int b = 4;
    return a * b;
}

int test02() {
    int a = 3;
    int b = 4;
    return a * a * b == 36;
}

int test03() {
    int ice = 3;
    int cream = 7;
    return ice * cream + ice + cream;
}

int test04() {
    int a = 3;
    int c;
    int b = c = a;
    return a * b * c;
}

int test05() {
    int a = 47;
    int b = 19;
    return a + b;
    return a % b;
}

int test06() { if (3 * 6 > 15) return 5; else return 10; }

int test07() {
    int x = 5;
    if (x > 4) x = x + 5;
    return x - 3;
}

int test08() {
    int a = 3;
    while (a * a < 100) a = a + 3;
    return a;
}

int test09() {
    int a = 0;
    int i;
    for (i = 0; i < 10; i = i + 1) a = a + i;
    return a;
}

int test10() {
    int a = 0;
    for (;; a = a + 3) if (a >= 10) break;
    return a;
}

int test11() {
    int a = 0;
    int i;
    int j;
    for (i = 0; i < 10; i = i + 1) {
        j = 0;
        while (j < 5) {
            a = a + 1;
            j = j + 1;
        }
    }
    return a;
}

int test12() {
    int i;
    for (i = 0; i < 1;) { break; }
    return i;
}

int test13() {
    int a = 0;
    int j = 0;
    while (1) {
        if (j >= 5) break;
        a = a + j;
        j = j + 1;
    }
    return a;
}

int test14() {
    int a = 0;
    int i;
    for (i = 0; i < 10; i = i + 1) {
        int j = 0;
        while (1) {
            if (j >= 5) break;
            a = a + 1;
            j = j + 1;
        }
    }
    return a;
}

int test15() {
    int x;
    int *y;
    y = &x;
    *y = 3;
    return x;
}

int test16() { return sizeof(sizeof(1)); }

int test17() { return sizeof(8); }

int test18() {
    int *y;
    return sizeof(y);
}

int test19() {
    int *y;
    return sizeof *y;
}

int test20() {
    int a[3][12];
    return sizeof a;
}

int test21() {
    int a[2];
    *a = 1;
    *(a + 1) = 2;
    int *p;
    p = a;
    return *p + *(p + 1);
}

int test22() {
    int a[3];
    a[0] = 8;
    a[3] = 9;
    return a[0];
}

int test23() {
    int a[4];
    a[0] = 8;
    a[3] = 91;
    return 3[a];
}

int arr[100];
int c;

int test24() {
    c = 4;
    arr[10] = 5;
    arr[7] = 7;
    return c + arr[10];
}

int test25() {
    char x[3];
    x[0] = -100;
    x[1] = 2;
    int y;
    y = 4;
    return x[0] + y;
}

char x[3];

int test26() {
    x[0] = -1;
    x[1] = 2;
    int y;
    y = 4;
    return x[0] + y;
}

int arr2[100][100] = {{7, 8},
                      {11}};

int test27() { return arr2[0][0] * arr2[0][1] * arr2[1][0]; }

int eval_check = 15 * 44 + (51 - 24) % 19 + (9 < 3) * 11 + 17 * (3 >= 4) - 29 * (123 != 1);

int test28() { return eval_check; }

int test29() {
    int arr[3][3] = {{1, 2, 3},
                     {},
                     {17, 8}};
    return arr[0][0] * arr[0][1] * arr[0][2] * (arr[2][0] + arr[2][1]);
}

int test30() {
    int b = 11, arr[2][2][2] = {{{}, {0, 3}},
                                {{0, 7}}};
    return arr[0][1][1] * arr[1][0][1] * b;
}

int test31() {
    int s = 2;
    s += 5;
    s -= 4;
    s *= 7;
    return s;
}

int test32() {
    int r = 1200;
    int s = 299;
    s *= s %= 39; // Undefined Behavior
    r /= 2;
    return s - r;
}
int test32() {
    int r = 1200;
    int s = 299;
    s *= s %= 39; // Undefined Behavior
    r /= 2;
    return s - r;
}
int test33() {
    int r = 853634;
    int x = 543636;
    return r^x^x;
}

int test34() {
    int x = 50;
    if (x > 40 && x < 60) {
        x += 30;
    }
    if (x > 60 && x / 2 == 30) {
        x += 20;
    }
    if (x < -100 || x < 100 && x%10 == 0) {
        x += 40;
    }
    return x;
}



int main() {
    assert(104, add6(Add(3, 8), 2, 3, 4, 5, 6));
    assert(12, test01());
    assert(1, test02());
    assert(31, test03());
    assert(27, test04());
    assert(66, test05());
    assert(5, test06());
    assert(7, test07());
    assert(12, test08());
    assert(45, test09());
    assert(12, test10());
    assert(50, test11());
    assert(0, test12());
    assert(10, test13());
    assert(50, test14());
    assert(3, test15());
    assert(4, test16());
    assert(4, test17());
    assert(8, test18());
    assert(4, test19());
    assert(144, test20());
    assert(3, test21());
    assert(8, test22());
    assert(91, test23());
    assert(9, test24());
    assert(-96, test25());
    assert(3, test26());
    a = -3;
    assert(1836311903, fib(46));
    assert(616, test27());
    assert(639, test28());
    assert(150, test29());
    assert(231, test30());
    assert(21, test31());
    assert(76, test32());
    assert(853634, test33());
    assert(120, test34());
    return 0;
}//