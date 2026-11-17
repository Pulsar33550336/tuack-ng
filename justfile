# JustFile for Tuack-NG

# 初始化项目所需信息
init:
    # Init git submodule
    git submodule update --init --recursive
    # 提交信息校验钩子（.githooks/commit-msg）
    git config core.hooksPath .githooks

# 构建 Checker
build-checkers:
    @cargo xtask build-checkers

# 构建项目
build: build-checkers
    @cargo build

# 以发布模式构建
build-release: build-checkers
    @cargo build --release

# 构建发布产物
dist platform *args:
    @cargo xtask dist --platform {{platform}} {{args}}

# 运行
[no-cd]
run +args="": build-checkers
    @cargo run -- {{ args }}

[working-directory("assets/templates")]
unwrap:
    @cargo run -- develop unwrap

[working-directory("assets/templates")]
wrap:
    @cargo run -- develop wrap
