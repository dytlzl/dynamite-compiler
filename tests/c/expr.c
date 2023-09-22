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
    assert(1, 5 - 3 == 2);
    assert(1, 123 < 31 * 4);
    assert(1, 124 <= 31 * 4);
    assert(0, 124 > 31 * 4);
    assert(1, 124 >= 31 * 4);
    int b, c = 4, *d, e[3];
    e[2] = 5;
    b = 3;
    d = &e[2];
    assert(60, b * c * *d);
    assert(1, 5&3);
    assert(6, 5^3);
    assert(7, 5|3);
    assert(142857, 142857^37504375^37504375);
    assert(5121, 5435&-8575);
    assert(2137, (855<<1)+(855>>1));
    assert(-3527, (-855<<2)+(-855>>3));
    assert(-124, ~123);
    assert(0, !(123 < 234));
    assert(1, !0);
    assert(5, 3*8 > 14 ? 3+2 : 4-2);
    assert(8, 3*8 > 34 ? 3*2 : 4*2);
}
