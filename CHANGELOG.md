# 更新日志

## v2026.9.19.0

1. 以 Tauri 2 初始化项目：Rust 后端 + React/TypeScript/Tailwind CSS/shadcn/ui 前端（Vite 构建，pnpm 管理依赖）
2. 电源计划通过 powrprof.dll 原生 API 实现读取、切换、复制与恢复默认（PowerEnumerate/PowerGetActiveScheme/PowerSetActiveScheme/PowerDuplicateScheme/PowerRestoreDefaultPowerSchemes），全程普通用户权限
3. 主页面：电源计划列表单选切换、卓越性能计划卡片（缺失时提供创建入口）、电源选项入口、状态（当前计划与时间，定时器仅在页面可见时运行）
4. 托盘：Tauri 2 内置 tray-icon，菜单由当前快照生成（计划列表勾选当前计划、开机自启动开关、打开主窗口、退出），文案经后端 Fluent 本地化
5. 开机自启动：tauri-plugin-autostart（注册表 HKCU Run 项附带 --silent 静默参数）；启动到托盘；关闭主窗口时托盘启用则隐藏保活
6. 单实例检测：tauri-plugin-single-instance，重复启动聚焦已存在实例
7. 设置页：语言（切换即时生效，无需重启）、开机自启动、启用托盘、启动到托盘、恢复电源计划（需管理员，成功后清空储存的卓越性能 UUID）、官网与代码仓库入口、版本号
8. 设置持久化：tauri-plugin-store（应用数据目录 settings.json），旧 WinUI 版 LocalSettings 数据不迁移
9. 多语言：前端 i18next、后端 Fluent（fluent-templates），沿用旧版 resw 全部 7 种语言文案（简体中文、繁体中文、英语、法语、意大利语、德语、西班牙语）；界面使用 shadcn/ui 设计，不仿制 Windows 系统应用
10. 兼容无可读名称的内置电源计划（如平衡）：PowerReadFriendlyName 双返回码处理，空名回退显示 GUID 文本
11. 电源计划行为对齐旧版 WinUI 3 实现：枚举单次缓冲调用、空名计划回退 GUID 文本、复制计划（先校验名称后创建副本）、计划列表 5 分钟缓存与写操作失效、卓越性能卡片三态（激活失败清空储存 UUID）、托盘三行式提示、错误消息两级渲染
12. 修正旧版文案数据 bug：PowerPlan.Error.Win32 占位符重复（{0}：{0} → {0}：{1}）
13. GitHub Actions 打包 x64 与 arm64 msixbundle，沿用旧版微软商店应用身份（BlazeSnow.PowerPlan）
