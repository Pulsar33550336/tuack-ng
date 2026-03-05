import sys

def main():
    sys.stdin = open("candy.in", "r")
    sys.stdout = open("candy.out", "w")

    n, m = map(int, input().split())
    C = [0] * (n + 1)
    h = 10**18  # 相当于C++中的1e18
    s = 0

    for i in range(1, n + 1):
        x, y = map(int, input().split())
        C[i] = x
        h = min(h, x + y)

    C.sort()  # 排序，注意C[0]是0
    # 前缀和
    for i in range(1, n + 1):
        C[i] += C[i - 1]

    for i in range(n + 1):
        if m < C[i]:
            break
        s = max(s, i + (m - C[i]) // h * 2)

    print(s)

if __name__ == "__main__":
    main()