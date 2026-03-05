#include "testlib.h"
#include <iostream>
#include <string>

using namespace std;

int main(int argc, char *argv[]) {
    // 注册命令行参数
    registerGen(argc, argv, 1);

    int nmin = opt<int>("nmin", 5); // 默认 n=5
    int nmax = opt<int>("nmax", 5); // 默认 n=5
    int n = rnd.next(nmin, nmax);
    long long seed = opt<unsigned long long>("seed", 0); // 种子，默认为 0
    rnd.setSeed(seed);

    cout << n << endl;

    for (int i = 1; i <= n; ++i) {
        cout << rnd.next(-(int)1e9, (int)1e9) << " ";
    }

    return 0;
}