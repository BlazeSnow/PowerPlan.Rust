# 设置与持久化

设置页面与持久化存储约定。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

## 设置页面

1. 设置界面使用shadcn/ui组件设计
2. 语言（下拉框）：切换显示语言，更改后即时生效，多语言架构见[语言](./i18n.md)
3. 开机自启动（开关）：默认为关；开关行下方显示系统侧实际状态（双模式机制见[开机自启动](./tray.md)），unsupported时禁用开关
4. 启用托盘（开关）：默认为开
5. 启动到托盘（开关）：默认为关，需先启用托盘，启动时直接进入托盘不显示主窗口
6. 恢复电源计划（按钮）：恢复电源计划到默认状态`PowerRestoreDefaultPowerSchemes`，成功后清空储存的卓越性能计划UUID（见[卓越性能计划存在性](./power-plans.md)）；权限要求见[架构与权限](./architecture.md)
7. 开发者官网（按钮）：<https://www.blazesnow.com>
8. 代码仓库（按钮）：<https://github.com/BlazeSnow/PowerPlan.Rust>
9. 软件版本号：显示当前安装包版本，从Tauri应用配置读取，不硬编码版本号

## 持久化设置

1. 设置内容通过`tauri-plugin-store`保存为JSON文件（应用数据目录）
2. 旧WinUI版（LocalSettings）的数据不迁移：两者包身份不同，无法跨包读取；卓越性能UUID缺失时按[卓越性能计划识别逻辑](./power-plans.md)重新处理
3. 存储字段：
   1. 语言：`language`
   2. 开机自启动：`auto_start_enabled`
   3. 启用托盘：`tray_enabled`
   4. 启动到托盘：`launch_to_tray`
   5. 卓越性能计划UUID：`ultimate_performance_plan_guid`
