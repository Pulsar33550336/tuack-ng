#!/usr/bin/env python
# -*- coding: utf-8 -*-
# tuack/cli.py

import sys
import runpy

def main():
    if len(sys.argv) < 2 or sys.argv[1] in ['-h', '--help']:
        print("""
tuack - OI/ICPC 题目管理工具

用法: tuack <command> [args...]

命令:
    gen, test, ren, dump, load, doc, install
        """)
        return

    command = sys.argv[1]
    
    modules = {
        'gen': 'tuack.gen',
        'test': 'tuack.test',
        'ren': 'tuack.ren',
        'dump': 'tuack.dump',
        'load': 'tuack.load',
        'doc': 'tuack.doc',
        'install': 'tuack.install',
    }
    
    if command not in modules:
        print(f"错误: 未知命令 '{command}'")
        sys.exit(1)
    
    sys.argv = [modules[command]] + sys.argv[2:]
    
    try:
        runpy.run_module(modules[command], run_name='__main__')
    except SystemExit:
        raise
    except Exception as e:
        print(f"执行 '{command}' 时出错: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == '__main__':
    main()