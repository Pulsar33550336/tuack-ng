import java.util.*;
import java.io.*;

public class Main {
    static class FastIO {
        BufferedReader br;
        StringTokenizer st;
        PrintWriter out;

        public FastIO() {
            br = new BufferedReader(new InputStreamReader(System.in));
            out = new PrintWriter(System.out);
        }

        String next() {
            while (st == null || !st.hasMoreElements()) {
                try {
                    st = new StringTokenizer(br.readLine());
                } catch (IOException e) {
                    e.printStackTrace();
                }
            }
            return st.nextToken();
        }

        int nextInt() {
            return Integer.parseInt(next());
        }

        long nextLong() {
            return Long.parseLong(next());
        }

        double nextDouble() {
            return Double.parseDouble(next());
        }

        String nextLine() {
            String str = "";
            try {
                str = br.readLine();
            } catch (IOException e) {
                e.printStackTrace();
            }
            return str;
        }

        void print(Object obj) {
            out.print(obj);
        }

        void println(Object obj) {
            out.println(obj);
        }

        void flush() {
            out.flush();
        }

        void close() {
            try {
                br.close();
                out.close();
            } catch (IOException e) {
                e.printStackTrace();
            }
        }
    }

    public static void main(String[] args) throws IOException {
        // 设置文件输入输出
        System.setIn(new FileInputStream("candy.in"));
        System.setOut(new PrintStream(new FileOutputStream("candy.out")));

        FastIO io = new FastIO();

        int n = io.nextInt();
        long m = io.nextLong();
        long[] C = new long[n + 1];
        long h = Long.MAX_VALUE;
        long s = 0;

        for (int i = 1; i <= n; ++i) {
            long x = io.nextLong();
            long y = io.nextLong();
            C[i] = x;
            h = Math.min(h, x + y);
        }

        // 排序从下标1到n的部分
        Arrays.sort(C, 1, n + 1);

        // 计算前缀和
        for (int i = 1; i <= n; ++i) {
            C[i] += C[i - 1];
        }

        for (int i = 0; i <= n; ++i) {
            if (m < C[i]) break;
            s = Math.max(s, i + (m - C[i]) / h * 2);
        }

        io.println(s);
        io.flush();
        io.close();
    }
}