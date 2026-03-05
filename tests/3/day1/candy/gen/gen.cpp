#include "testlib.h"
#include <iostream>
#include <string>

using namespace std;

int main(int argc, char* argv[])
{
    // 注册命令行参数
    registerGen(argc, argv, 1);

    // 从命令行获取参数
    // 格式: ./generator n=value m=value x_max=value y_max=value seed=value
    // 所有参数都是可选的，有默认值
    int n = opt<int>("n", 5); // 默认 n=5
    long long m = opt<long long>("m", 20); // 默认 m=20
    int x_max = opt<int>("x_max", 1000000000); // 默认 x_i 最大值 10
    int y_max = opt<int>("y_max", 1000000000); // 默认 y_i 最大值 10
    long long seed = opt<unsigned long long>("seed", 0); // 种子，默认为 0

    // 使用种子初始化随机数生成器
    rnd.setSeed(seed);

    // 输出 n 和 m
    cout << n << " " << m << endl;

    // 输出 n 行，每行两个整数 x_i, y_i
    for (int i = 0; i < n; i++) {
        // 在 [1, x_max] 范围内生成 x_i
        int x = rnd.next(1, x_max);
        // 在 [1, y_max] 范围内生成 y_i
        int y = rnd.next(1, y_max);
        cout << x << " " << y << endl;
    }

    return 0;
}