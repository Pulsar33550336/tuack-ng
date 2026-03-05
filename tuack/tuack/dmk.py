help = u'''\
用于造数据，把gen.py放在problem目录下，然后运行：
python -m tuack.dmk [开关]
具体写法在gen.py有相关说明。
-s i,compile,g,run  跳过若干个步骤不执行。
                    i,init：初始化函数
                    c,compile：编译标程
                    g,gen：生成数据
                    r,run：运行标程
'''

from .base import log
from . import base
import yaml, os, time, sys, random

folders = ['down', 'data']
fmap = {'down' : 'samples', 'data' : 'data'}

class Getter:
	def __init__(self, arg1, arg2):
		self.arg1 = arg1
		self.arg2 = arg2
	def __getitem__(self, key):
		return self.arg1.get(key, self.arg2.get(key, None))

def main(argv):
	base.init()
	skip = set()
	flag = None
	for arg in argv:
		if arg == '-h':
			log.info(help)
			return
		elif flag == '-s':
			for key in arg.split(','):
				skip.add({
					'i' : 'init',
					'c' : 'compile',
					'g' : 'gen',
					'r' : 'run'
				}.get(key, key))
			flag = None
		elif flag:
			log.error(f"错误的参数{flag}。")
			return
		elif arg.startswith('-'):
			flag = arg
		else:
			log.info(f"未知的传入信息{arg}。")
	import gen, platform
	if 'init' not in skip:
		if hasattr(gen, 'init'):
			gen.init()
		else:
			log.info('没有init函数，跳过执行。')
	conf = base.conf
	if 'compile' not in skip:
		if not hasattr(gen, 'std'):
			log.info('没有std变量，将查找第一个expect能取到100的标程。')
			gen_std = None
			for user, codes in conf.get('users').items():
				for code, info in codes.items():
					if eval('100 ' + info.get('expected', '')) == True:
						gen_std = (user, code)
						break
				if gen_std: break
			if not gen_std: log.warning('没有找到任何可能像标程的代码，请配置conf.yaml的users字段。')
		else:
			gen_std = gen.std
		if gen_std:
			os.system('g++ %s -o std.exe -std=c++17 -O2 -Wall -Wextra %s' % (
				conf.get('users')[gen_std[0]][gen_std[1]].get('path'),
				f"-Wl,--stack={int(conf.ml().B)}" if platform.system() == 'Windows' else ''
			))
	std_exe = False
	if 'run' not in skip:
		if os.path.isfile('std.exe'):
			std_exe = True
		else:
			log.warning('没有std.exe，无法生成输出文件。')
	if os.path.isfile('random_state'):
		random.setstate(eval(open('random_state').read()))
	else:
		open('random_state', 'w').write(str(random.getstate()))
	warnend_gen_gen = False
	for folder in folders:
		os.makedirs(folder, exist_ok=True)
		for item in conf[fmap[folder]]:
			for case in item['cases']:
				log.info(f'数据类型{folder}，测试点{case}。')
				if hasattr(gen, 'gen'):
					if 'gen' not in skip and not item['args'].get('manual'):
						with open(f'{folder}/{case}.in', 'w') as f:
							gen.gen(
								f,
								Getter(item.get('args', {}), conf.get('args', {})),
								{
									'case' : case,
									'folder' : folder
								}
							)
				elif not warnend_gen_gen:
					log.warning('没有数据生成函数gen，跳过数据生成。')
					warnend_gen_gen = True
				if std_exe:
					t = time.perf_counter()

					if base.system == 'Windows':
						os.system(f'std.exe < {folder}/{case}.in > {folder}/{case}.ans')
					elif base.system == "Linux":
						os.system(f'./std.exe < {folder}/{case}.in > {folder}/{case}.ans')

					log.info(f"标程用时：{time.perf_counter() - t}")
					ans = ' '.join([line.strip() for line in open(f'{folder}/{case}.ans')])
					if len(ans) > 100: 
						ans = ans[:100] + f"（省略{len(ans) - 100}个字符。）"
					log.info('答案：' + ans)

if __name__ == '__main__':
	main(sys.argv[1:])
