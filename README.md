# Jobs
大学生玩具项目，扫盘，记录文件夹信息，持久化，命令行交互。

使用了设计模式，借鉴了许多黑魔法，测试驱动开发，代码质量尚可。

扫盘5000文件花费约3s，时间指数级上升，需要优化。

下一个feat不会是大版本，而是新的序列化方法，加速读取和分块存储。

目前仅仅是能正常运行的程度，瞎搞会直接panic死掉。

# Usage
直接运行开启命令行交互
```sh
Jobs.exe
```
```
[Jobs]@E:\1-code\__repo__\Jobs >> 
```

目前接受的命令：
- `cd` 进入指定目录
- `ls` 列出目录下的文件信息
- `scan` 扫描当前目录
- `show` 查看当前目录状态信息 
- `tree` 查看当前目录树状结构
- `dump` 保存至用户根目录/example.csv
- `load` 从用户根目录/example.csv加载
- `quit` 优雅退出
- UP / DOWN 切换历史命令
- TAB 同ls

# Other Stuff

## Aiming
- [x] Version 1
  - [x] Scan disk and calculate folder size, reusing existing fsnodes.
  - [x] Serialize and Deserialize
  - [x] Console interaction
- [ ] Version 2
  - [ ] Fuzz search, prefix and suffix trie
  - [ ] Cross platform
  - [ ] Python interface
- [ ] Version 3
  - [ ] synchronize with remote server

## Difficulties
- [ ] How to gracefully perform console interaction?
  - Answer: `crossterm`, a cross-platform library for terminal control.
- [ ] How to serialize and deserialize?
  - Answer: `serde`, deprecate JSON, use CSV instead.

## Debug
```sh
cargo run --release --bin project -- --debug
cargo test test_mng
cargo check
```
