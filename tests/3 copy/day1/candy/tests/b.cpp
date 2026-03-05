#include <bits/stdc++.h>

#define FstIO ios::sync_with_stdio(0), cin.tie(0), cout.tie(0)
#define pii pair<ll, ll>

using namespace std;
using ll = long long;

const ll N = 5e5 + 5, M = 320;

ll n, m;
ll C[N], h = 1e18, s = 0;

signed main()
{
    freopen("candy.in", "r", stdin);
    freopen("candy.out", "w", stdout);

    FstIO;

    cin >> n >> m;
    for (ll i = 1; i <= n; ++i) {
        ll x, y;
        cin >> x >> y;
        C[i] = x;
        h = min(h, x + y);
    }
    sort(C + 1, C + n + 1);
    for (ll i = 1; i <= n; ++i)
        C[i]
            += C[i - 1];
    for (ll i = 0; i <= n; ++i) {
        if (m < C[i])
            break;
        s = max(s, i + (m - C[i]) / h * 2);
    }
    cout << s << '\n';

    return 0;
}
