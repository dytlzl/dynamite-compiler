#include <stdio.h>

int value;
char *fmt2;
char *fmt3 = "a = %d, ";
int a[3] = {3, 5, 7};

int main() {
    value = 777;
    printf("a = %d, ", value);
    char *fmt = "b = %d, ";
    value = 755;
    printf(fmt, value);
    fmt2 = "c = %d\n";
    printf(fmt2, 222);
    return 0;
}
