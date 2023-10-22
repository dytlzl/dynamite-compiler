#include <stdlib.h>
#include <stdio.h>

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
int main() {
    assert(0, 0);
    assert(42, 42);
    assert(-7, -7);
    assert(13, 3 * (29 / (13 - 2) + 3) - 2);
    assert(28, 3 * (29 % (13 - 2) + 3) - 2);
    assert(10, -1 * 4 + 2 * +7);
    assert(1, 5&3);
    assert(6, 5^3);
    assert(7, 5|3);
    assert(142857, 142857^37504375^37504375);
    assert(5121, 5435&-8575);
    return 0;
}
