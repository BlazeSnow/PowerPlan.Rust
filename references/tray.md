# 托盘与应用生命周期

托盘、开机自启动与单实例约定。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

## 托盘

1. 使用Tauri 2内置托盘（tray-icon）实现，图标使用应用图标（`src-tauri/icons`下的`.ico`）
2. 托盘菜单每次打开由当前快照生成，结构与顺序对齐旧版TrayMenuBuilder：禁用标题项（应用标题）、打开主窗口、电源计划列表（当前计划使用勾选标识）、隐藏的卓越性能（仅当储存UUID存在且不在列表时显示，激活失败清空UUID）、刷新计划（强制刷新缓存）、开机自启动切换（无勾选状态，文案"开启/关闭"即状态）、退出
3. 菜单图标以Unicode字形前缀拼入文本（⌂/⚡/↻/⏻/✕，对齐旧版TrayMenuBuilder），单色渲染随菜单深浅色自适应；标题项不加；不使用muda图标槽（旧版经验：会挤压文本，且深浅色需双套位图）
3. 托盘菜单深浅色跟随系统应用主题：启动时经uxtheme兼容层（`tray_theme.rs`，1903+用`SetPreferredAppMode(AllowDark)`，1809–1902用`AllowDarkModeForApp`）开启应用级深色策略，动态重建菜单后刷新沉浸式颜色策略与菜单主题缓存；不支持的系统或API解析失败时回退默认策略；禁止`ForceDark`、`ForceLight`或硬编码菜单颜色
4. 关闭主窗口时若托盘启用：保存窗口几何后**销毁webview**（内存随webview进程退出释放），应用保活；销毁后的重建（`ensure_main_window`）在独立线程按tauri.conf.json配置执行（tauri派发回主线程创建），并做`AtomicBool`防重入——**不得在主线程消息处理（单实例WM_COPYDATA、托盘菜单点击）中同步重建**，WebView2创建需泵消息，嵌套等待会死锁
5. 托盘与主页面的计划状态保持同步：托盘切换计划后通知前端；主窗口销毁期间无需同步，重建挂载时自动拉取最新状态
6. 退出：先销毁托盘与菜单资源，再退出应用
7. Explorer重启后托盘图标需恢复（tray-icon库已内置`TaskbarCreated`处理，升级依赖后需回归验证）
8. 修改托盘相关实现后，必须验证：动态菜单刷新、系统浅深主题、静默启动、重复打开菜单、Explorer重启恢复和退出流程
9. 托盘提示为三行式（对齐旧版TrayTooltipFormatter）：应用标题（读自tauri.conf.json的productName）、当前计划、自启动开关状态；超长截断至127字符
10. 菜单与提示的计划快照来自带缓存的列表（见[计划列表缓存](./power-plans.md)），托盘切换计划后需使缓存失效、重建菜单并通知前端刷新

## 开机自启动

1. 双模式适配（`src-tauri/src/autostart.rs`）：
   1. MSIX 打包版：`StartupTask`（WinRT，TaskId=`PowerPlanStartupTask`，须与msix/AppxManifest.template.xml的uap5声明一致）；MSIX下注册表Run被虚拟化不可用；对齐旧版StartupService
   2. 未打包（开发构建）：`tauri-plugin-autostart`（注册表HKCU Run项，附带静默参数）
2. 静默启动依据：`--silent`参数（未打包），或`GetActivatedEventArgs().Kind == StartupTask`（打包版登录激活）
3. 设置页开关行不显示状态提示（开关位置即状态）；`settings_get`返回`autoStartState`（enabled/disabled/disabled_by_user/disabled_by_policy/unsupported）仅用于unsupported时禁用开关
4. 打包版启用被系统拒绝（常见：用户曾在任务管理器/启动设置中禁用）时提示前往系统设置重新开启
5. 用户从托盘菜单点击"打开主窗口"时，显示主窗口并聚焦
6. 主窗口默认隐藏创建，仅在需要时显示，使静默启动无需特殊分支

## 开机自启动本地测试

1. 正常启动应用，开启"开机自启"和"托盘"设置，关闭应用
2. 以静默参数启动构建产物，模拟登录触发：

   ```powershell
   .\src-tauri\target\release\powerplan.exe --silent
   ```

3. 预期：窗口不弹出，图标直接进入托盘
4. 真实链路验证：登出再登入即可，不需要重启电脑

## 单实例

1. 使用`tauri-plugin-single-instance`实现单实例检测
2. 重复启动时聚焦已存在的实例：主窗口隐藏时将其显示并聚焦
