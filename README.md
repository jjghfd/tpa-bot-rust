# TPA Bot - Rust版本

一个基于Rust和Azalea框架的Minecraft机器人，提供传送、权限管理等功能。

## 功能特性

- 玩家位置查询（通过BlueMap API）
- 传送功能（tpa me/tpa you）
- 权限管理系统
- 设置传送点
- 挖矿模式

## 配置

编辑 `config.toml` 文件：

```toml
[bot]
username = "你的机器人用户名"
server_address = "服务器地址"
is_offline = true

[bluemap]
api_url = "BlueMap API地址"
```

## 构建

```bash
cargo build --release
```

## 使用方法

在游戏中输入以下指令：

- `%开盒 [玩家名字]` - 查询玩家位置（管理员）
- `%tpa me` - 让机器人传送到你这里（管理员）
- `%tpa you` - 让你传送到机器人那里（管理员）
- `%挖矿` - 机器人开始挖矿（管理员）
- `%设置传送点 [名字]` - 传送并设置家（管理员）
- `%op [玩家名字]` - 添加管理员（超级管理员）
- `%deop [玩家名字]` - 移除管理员（超级管理员）
- `%op查询` - 查询管理员列表（管理员）

## 依赖

- Rust 1.70+
- Azalea Minecraft客户端框架
- Tokio异步运行时