#include <bits/stdc++.h>
#define int long long
using namespace std;

priority_queue<int> q;

signed main() {
    freopen("CF13C.in", "r", stdin);
    freopen("CF13C.out", "w", stdout);
    // while (1)
    //     ;
    int n;
    cin >> n;
    int ans = 0;
    for (int i = 1; i <= n; ++i) {
        int x;
        cin >> x;
        q.push(x);
        if (x < q.top()) {
            ans += q.top() - x;
            q.pop();
            q.push(x);
        }
    }
    cout << ans;
}
