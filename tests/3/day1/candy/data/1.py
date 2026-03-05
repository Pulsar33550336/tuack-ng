import json
import matplotlib.pyplot as plt
import numpy as np
from scipy import stats
import seaborn as sns
from collections import Counter
import hashlib
import os
from tqdm import tqdm
import math

def read_json_file(file_path, max_samples=None):
    """读取JSON文件，支持大文件分批读取"""
    values = []
    
    try:
        with open(file_path, 'r') as f:
            data = json.load(f)
        
        # 如果是字典，提取值
        if isinstance(data, dict):
            print(f"读取到 {len(data)} 个键值对")
            values = list(data.values())
        elif isinstance(data, list):
            print(f"读取到 {len(data)} 个元素的列表")
            values = data
        else:
            print(f"数据类型: {type(data)}")
            values = [data]
        
        # 限制样本数量
        if max_samples and len(values) > max_samples:
            print(f"采样 {max_samples} 个值进行分析...")
            indices = np.random.choice(len(values), max_samples, replace=False)
            values = [values[i] for i in indices]
            
        return values
        
    except json.JSONDecodeError as e:
        print(f"JSON解析错误: {e}")
        return []
    except Exception as e:
        print(f"读取文件错误: {e}")
        return []

def analyze_randomness(values):
    """分析数值的随机性"""
    if not values:
        print("没有数据可分析")
        return
    
    print(f"\n=== 随机性分析报告 ===")
    print(f"样本数量: {len(values)}")
    
    # 转换为numpy数组
    arr = np.array(values, dtype=np.uint64)
    
    # 1. 基本统计
    print(f"\n1. 基本统计:")
    print(f"   最小值: {arr.min()}")
    print(f"   最大值: {arr.max()}")
    print(f"   平均值: {arr.mean():.2f}")
    print(f"   标准差: {arr.std():.2f}")
    print(f"   中位数: {np.median(arr)}")
    
    # 2. 分布均匀性检验
    print(f"\n2. 分布均匀性检验:")
    
    # 卡方检验（将64位值分为16个区间）
    hist, bins = np.histogram(arr, bins=16)
    chi2, p_value = stats.chisquare(hist)
    print(f"   卡方检验 p-value: {p_value:.6f}")
    print(f"   {'✓' if p_value > 0.05 else '✗'} {'通过' if p_value > 0.05 else '未通过'}均匀性检验 (α=0.05)")
    
    # 3. 随机性检验
    print(f"\n3. 随机性检验:")
    
    # 转换为二进制字符串进行游程检验
    binary_strings = [format(x, '064b') for x in arr[:1000]]  # 限制样本数量
    combined_bits = ''.join(binary_strings)
    
    # 游程检验（Run Test）
    runs = 1
    for i in range(1, len(combined_bits)):
        if combined_bits[i] != combined_bits[i-1]:
            runs += 1
    
    n = len(combined_bits)
    n1 = combined_bits.count('1')
    n0 = combined_bits.count('0')
    
    expected_runs = 2 * n1 * n0 / n + 1
    variance_runs = (2 * n1 * n0 * (2 * n1 * n0 - n)) / (n * n * (n - 1))
    z_score = (runs - expected_runs) / np.sqrt(variance_runs)
    p_runs = 2 * (1 - stats.norm.cdf(abs(z_score)))
    
    print(f"   游程检验 p-value: {p_runs:.6f}")
    print(f"   {'✓' if p_runs > 0.05 else '✗'} {'通过' if p_runs > 0.05 else '未通过'}随机性检验")
    
    # 4. 自相关性检验
    print(f"\n4. 自相关性检验:")
    if len(arr) > 1:
        correlation = np.corrcoef(arr[:-1], arr[1:])[0, 1]
        print(f"   相邻值相关系数: {correlation:.6f}")
        print(f"   {'✓' if abs(correlation) < 0.1 else '✗'} 相关性{'较弱' if abs(correlation) < 0.1 else '较强'}")
    
    # 5. 熵估计
    print(f"\n5. 信息熵分析:")
    byte_data = arr.tobytes()
    freq = Counter(byte_data)
    total = len(byte_data)
    entropy = -sum((f/total) * math.log2(f/total) for f in freq.values())
    max_entropy = math.log2(256)  # 8 bits per byte
    entropy_ratio = entropy / max_entropy
    print(f"   字节级熵值: {entropy:.4f} bits/byte")
    print(f"   最大可能熵: {max_entropy:.4f} bits/byte")
    print(f"   熵比: {entropy_ratio:.4f}")
    print(f"   {'✓' if entropy_ratio > 0.95 else '✗'} 熵值{'较高' if entropy_ratio > 0.95 else '较低'}")
    
    return arr

def create_visualizations(arr):
    """创建可视化图表"""
    if len(arr) == 0:
        return


    # import matplotlib
    # font_list = matplotlib.font_manager.fontManager.ttflist
    # for font in font_list:
    #     if ('Noto Sans CJK' in font.name) or ('Hei' in font.name):
    #         print(font.name)

    # return


    # plt.rcParams['font.sans-serif'] = ["Noto Sans CJK SC"]
    plt.rcParams['font.family'] = ['WenQuanYi Zen Hei']
    plt.rcParams['axes.unicode_minus'] = False
    
    plt.figure(figsize=(15, 10))
    
    # 1. 直方图
    plt.subplot(2, 3, 1)
    plt.hist(arr, bins=50, alpha=0.7, edgecolor='black')
    plt.title('数值分布直方图')
    plt.xlabel('数值')
    plt.ylabel('频次')
    plt.grid(True, alpha=0.3)
    
    # 2. 分位数图 (Q-Q Plot)
    plt.subplot(2, 3, 2)
    stats.probplot(arr, dist="uniform", plot=plt)
    plt.title('Q-Q图 (均匀分布检验)')
    plt.grid(True, alpha=0.3)
    
    # 3. 散点图（相邻值）
    plt.subplot(2, 3, 3)
    if len(arr) > 1:
        plt.scatter(arr[:-1], arr[1:], alpha=0.5, s=10)
        plt.title('相邻值散点图')
        plt.xlabel('X[i]')
        plt.ylabel('X[i+1]')
        plt.grid(True, alpha=0.3)
    
    # 4. 自相关图
    plt.subplot(2, 3, 4)
    if len(arr) > 20:
        autocorr = [np.corrcoef(arr[:-i], arr[i:])[0,1] for i in range(1, 21)]
        plt.bar(range(1, 21), autocorr)
        plt.title('自相关图 (前20个滞后)')
        plt.xlabel('滞后')
        plt.ylabel('相关系数')
        plt.grid(True, alpha=0.3)
    
    # 5. 比特位分布
    plt.subplot(2, 3, 5)
    bit_counts = np.zeros(64)
    for val in arr:
        for i in range(64):
            if val & (1 << i):
                bit_counts[i] += 1
    bit_prob = bit_counts / len(arr)
    plt.bar(range(64), bit_prob, alpha=0.7)
    plt.axhline(y=0.5, color='r', linestyle='--', alpha=0.5)
    plt.title('比特位为1的概率')
    plt.xlabel('比特位')
    plt.ylabel('概率')
    plt.grid(True, alpha=0.3)
    
    # 6. 3D散点图（连续三个值）
    plt.subplot(2, 3, 6)
    if len(arr) > 2:
        from mpl_toolkits.mplot3d import Axes3D
        ax = plt.gcf().add_subplot(2, 3, 6, projection='3d')
        ax.scatter(arr[:-2], arr[1:-1], arr[2:], alpha=0.5, s=10)
        ax.set_title('三维序列图')
        ax.set_xlabel('X[i]')
        ax.set_ylabel('X[i+1]')
        ax.set_zlabel('X[i+2]')
    
    plt.tight_layout()
    plt.show()
    
    # 7. 额外的：热力图展示字节值分布
    plt.figure(figsize=(12, 6))
    # 将u64转换为8个字节
    bytes_list = []
    for val in arr:
        bytes_list.extend(list(int(val).to_bytes(8, 'little')))
    
    # 创建16x16的字节值热力图
    heatmap = np.zeros((16, 16))
    for byte_val in bytes_list:
        high = byte_val >> 4
        low = byte_val & 0x0F
        heatmap[high, low] += 1
    
    plt.subplot(1, 2, 1)
    plt.imshow(heatmap, cmap='hot', interpolation='nearest')
    plt.colorbar(label='频次')
    plt.title('字节值分布热力图')
    plt.xlabel('低4位')
    plt.ylabel('高4位')
    
    # 8. 累积分布函数图
    plt.subplot(1, 2, 2)
    sorted_vals = np.sort(arr)
    y = np.arange(1, len(sorted_vals) + 1) / len(sorted_vals)
    plt.plot(sorted_vals, y, 'b-', linewidth=2)
    plt.title('经验累积分布函数')
    plt.xlabel('数值')
    plt.ylabel('累积概率')
    plt.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.show()

def save_analysis_report(arr, output_file="randomness_report.txt"):
    """保存分析报告到文件"""
    with open(output_file, 'w') as f:
        f.write("=== 随机性分析报告 ===\n")
        f.write(f"样本数量: {len(arr)}\n")
        f.write(f"数据范围: [{arr.min()}, {arr.max()}]\n")
        f.write(f"平均值: {arr.mean():.2f}\n")
        f.write(f"标准差: {arr.std():.2f}\n\n")
        
        # 计算一些额外的统计量
        f.write("=== 详细统计 ===\n")
        f.write(f"变异系数: {(arr.std()/arr.mean()):.6f}\n")
        f.write(f"偏度: {stats.skew(arr):.6f}\n")
        f.write(f"峰度: {stats.kurtosis(arr):.6f}\n")
        
        # 测试均匀性
        f.write("\n=== 统计检验 ===\n")
        hist, bins = np.histogram(arr, bins=16)
        chi2, p_value = stats.chisquare(hist)
        f.write(f"卡方检验 p-value: {p_value:.6f}\n")
        
        # KS检验
        ks_stat, ks_p = stats.kstest(arr, 'uniform', 
                                    args=(arr.min(), arr.max()-arr.min()))
        f.write(f"KS检验 p-value: {ks_p:.6f}\n")
        
    print(f"\n分析报告已保存到: {output_file}")

def main():
    # 配置文件路径
    json_file = ".seed"  # 替换为你的JSON文件路径
    max_samples = 10000  # 最大分析样本数，设为None则分析所有数据
    
    print(f"正在读取文件: {json_file}")
    
    # 读取数据
    values = read_json_file(json_file, max_samples)
    
    if not values:
        print("没有读取到有效数据，请检查文件路径和格式")
        return
    
    # 分析随机性
    arr = analyze_randomness(values)
    
    # 创建可视化
    create_visualizations(arr)
    
    # 保存报告
    save_analysis_report(arr)
    
    print("\n=== 分析完成 ===")
    print("已生成:")
    print("  1. 多种统计检验结果")
    print("  2. 多种可视化图表")
    print("  3. 文本分析报告 (randomness_report.txt)")

if __name__ == "__main__":
    main()