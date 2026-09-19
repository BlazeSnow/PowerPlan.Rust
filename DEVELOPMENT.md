# 开发指南

PowerPlan 的详细开发指南。基本约定与内容速览见 [AGENTS.md](./AGENTS.md)。详细规范拆分在 [references/](./references/) 目录下，本文档作为索引，应随开发进度同步更新；每完成一项功能，同步更新 [CHANGELOG.md](./CHANGELOG.md) 与对应文档。

## 程序目的

快速切换Windows电源计划

## 文档索引

| 主题 | 要点 | 详情 |
| --- | --- | --- |
| 架构与权限 | Tauri 2（Rust后端 + shadcn/ui前端）；`powrprof.dll`原生API；普通用户权限运行 | [架构与权限](./references/architecture.md) |
| 电源计划 | 原生API读取与切换；卓越性能计划仅按GUID识别；隐藏计划与恢复处理 | [电源计划](./references/power-plans.md) |
| 托盘与生命周期 | Tauri内置托盘；开机自启动静默启动；单实例检测 | [托盘与生命周期](./references/tray.md) |
| 界面 | 主页面、侧边栏、自绘标题栏 | [界面](./references/ui.md) |
| 语言 | 前端i18n；跟随系统、回退英语；支持7种语言 | [语言](./references/i18n.md) |
| 设置与持久化 | 仿Windows 11设置页；`tauri-plugin-store`持久化 | [设置与持久化](./references/settings.md) |
| 通用约定 | UTF-8与GBK终端处理；托盘常驻态性能优先 | [通用约定](./references/conventions.md) |
| 发布 | GitHub Actions打包；x64与arm64 msixbundle；Microsoft Store | [发布](./references/release.md) |
