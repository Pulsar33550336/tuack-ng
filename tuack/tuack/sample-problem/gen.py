#整个程序运行前将会先调用这个函数，可以不写
def init():
	from tuack import base
	#base.conf里面是预先读取好的conf.yaml数据，并对其进行了一些预处理
	#注意这段代码会【覆盖】data字段
	base.conf['data'] = []
	#这里是某种生成data参数的代码
	for i in range(1, 20):
		base.conf['data'].append({
			'cases' : [f"{i}-{j + 1}" for j in range(round(i ** 1.1))],
			'args' : {'n' : 20, 'm' : i}
		})
	#修改完conf以后需要保存
	base.save_json(base.conf)

from tuack import dmk
#如果使用random库，dmk会自动保存第一次运行的种子信息，之后运行会生成相同的数据
#如果需要重新随机，删除random_state即可
from random import randint

#生成数据的函数，参数分别是输出文件对象、参数获取器、上下文信息
def gen(fin, args, cont):
	# cont中有一些上下文信息
	dmk.log.debug(f"当前数据类型：{cont['folder']}")
	n = args['n']	#自动查找参数，优先查找测试点中的，找不到找全局，再找不到则是None
	m = args['m']
	tp = (cont['case'] if type(cont['case']) == int else int(cont['case'][-1])) & 1
	#往fin中输出即可
	fin.write(f"{n} {m}\n")
	for i in range(m - tp):
		fin.write(f"{randint(1, n)} {randint(1, n)}\n")
	if tp:
		fin.write('1 1\n')
