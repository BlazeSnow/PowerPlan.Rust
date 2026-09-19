# 架构与权限

PowerPlan 的程序架构、项目结构与权限约定。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

## 程序架构

1. 程序使用Tauri 2架构：Rust后端 + Web前端（React + TypeScript + Tailwind CSS + shadcn/ui，Vite构建，pnpm管理依赖）
2. 电源计划的读取与切换通过`powrprof.dll`原生API实现（Rust侧使用官方`windows` crate调用），不调用`powercfg`命令行
3. 最终打包msixbundle发布至Microsoft store

## 项目结构

1. `src/`：前端页面与组件
2. `src-tauri/src/`：Rust后端
   1. 业务核心（电源计划）：不依赖Tauri运行时，可独立单元测试（`cargo test`）
   2. 托盘、设置、Tauri command等模块与业务核心分离
3. 前端与后端通过Tauri command通信，command定义集中管理
4. 根目录脚本：`run.ps1`（开发/发布运行，`-Release -Msix`打包）、`version.ps1`（package.json版本同步到Cargo.toml/tauri.conf.json/Cargo.lock，唯一来源是package.json）、`tag.ps1`（按版本打`v`标签并推送，触发发布工作流）；`scripts/make-msix.ps1`负责msixbundle打包

## 权限

1. 程序以普通用户权限运行，全程不申请管理员权限
2. 电源计划的读取、切换和创建卓越性能计划副本均通过`powrprof.dll`原生API实现，不需要管理员权限
3. 例外：设置页的恢复电源计划调用`PowerRestoreDefaultPowerSchemes`，按微软文档要求调用者必须是本地Administrators组成员；权限不足时操作失败，弹窗提示错误
