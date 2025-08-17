
__attribute__((section(".text.fixed_foobar_func"))) int foobar(int a) {
  return a * 2;
}

__attribute__((section(".text.fixed_main_func"))) int main() {
  int a = 1000;
  int b = 2000;
  int c = a + b;
  int d = foobar(c);
  return d - 1;
}
