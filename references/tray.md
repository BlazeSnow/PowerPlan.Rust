# 托盘与应用生命周期

托盘、开机自启动与单实例约定。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

## 托盘

1. 使用Tauri 2内置托盘（tray-icon）实现，图标使用应用图标（`src-tauri/icons`下的`.ico`）
2. 托盘菜单在每次点击图标（左/右键抬起）时先重建再手动弹出，结构对齐旧版TrayMenuBuilder并按需精简：禁用标题项（应用标题）、打开主窗口、电源计划列表（当前计划使用勾选标识）、隐藏的卓越性能（仅当储存UUID存在且不在列表时显示，激活失败清空UUID）、刷新计划（强制刷新缓存）、打开设置（显示主窗口并导航到设置页，见下方「打开设置」）、退出
3. 菜单弹出机制：系统自动弹出的是预构建菜单快照，外部改动（任务管理器/系统设置切换自启动）会显示延迟状态，因此关闭左键自动弹出（`show_menu_on_left_click(false)`）与右键自动弹出（tauri未暴露，经`with_inner_tray_icon`调内层tray-icon的`set_show_menu_on_right_click(false)`，升级tauri需回归验证），在托盘图标事件回调内重建菜单与提示后调用内层`show_menu()`弹出（`Shell_NotifyIconGetRect`屏幕坐标定位，任务栏溢出区同样有效）；回调运行在主线程，内层调用内联执行无跨线程等待
6. 仅电源计划项以⚡字形前缀拼入文本标识类目（含隐藏的卓越性能项）；其余菜单项纯文本——文本自明，字形图标跨字体渲染不一致且无信息增益；标题项不加；不使用muda图标槽（旧版经验：会挤压文本，且深浅色需双套位图）
5. 托盘不设自启动开关：菜单每次打开都重建，开关项会迫使每次点击都做系统状态查询（打包版为WinRT StartupTask调用，是菜单构建最重的一环）；自启动的显示与切换一律在设置页进行
6. 托盘菜单深浅色跟随系统应用主题：启动时经uxtheme兼容层（`tray_theme.rs`，1903+用`SetPreferredAppMode(AllowDark)`，1809–1902用`AllowDarkModeForApp`）开启应用级深色策略，动态重建菜单后刷新沉浸式颜色策略与菜单主题缓存；不支持的系统或API解析失败时回退默认策略；禁止`ForceDark`、`ForceLight`或硬编码菜单颜色
4. 关闭主窗口时若托盘启用：保存窗口几何后**销毁webview**（内存随webview进程退出释放），应用保活；销毁后的重建（`ensure_main_window`）在独立线程按tauri.conf.json配置执行（tauri派发回主线程创建），并做`AtomicBool`防重入——**不得在主线程消息处理（单实例WM_COPYDATA、托盘菜单点击）中同步重建**，WebView2创建需泵消息，嵌套等待会死锁
5. **启动到托盘/静默启动时不创建主窗口与webview**（tauri.conf.json窗口配置`create: false`，延迟创建）：托盘常驻零webview内存；打开主窗口时按需创建。重建后的几何恢复（`restore_state`）与显示必须经`run_on_main_thread`在主线程执行——从工作线程调用会跨线程等待主线程而死锁（build本身在工作线程安全）
6. 托盘常驻期间启用Windows效能模式（EcoQoS）：静默/启动到托盘、关闭主窗口时启用，主窗口打开时恢复全性能（见conventions.md「性能要求」）
7. 托盘与主页面的计划状态保持同步：托盘切换计划后通知前端；主窗口销毁期间无需同步，重建挂载时自动拉取最新状态
6. 退出：先销毁托盘与菜单资源，再退出应用
7. Explorer重启后托盘图标需恢复（tray-icon库已内置`TaskbarCreated`处理，升级依赖后需回归验证）
8. 修改托盘相关实现后，必须验证：动态菜单刷新、系统浅深主题、静默启动、重复打开菜单、Explorer重启恢复和退出流程
9. 托盘提示为两行式：应用标题（读自tauri.conf.json的productName）、当前计划；超长截断至127字符（旧版三行式的自启动行随托盘自启动开关一并移除，避免每次重建都做系统状态查询）
10. 菜单与提示的计划快照来自带缓存的列表（见[计划列表缓存](./power-plans.md)），托盘切换计划后需使缓存失效、重建菜单并通知前端刷新

## 开机自启动

1. 双模式适配（`src-tauri/src/autostart.rs`）：
   1. MSIX 打包版：`StartupTask`（WinRT，TaskId=`PowerPlanStartupTask`，须与msix/AppxManifest.template.xml的uap5声明一致）；MSIX下注册表Run被虚拟化不可用；对齐旧版StartupService
   2. 未打包（开发构建）：`tauri-plugin-autostart`（注册表HKCU Run项，附带静默参数）
2. **显示与切换一律以系统侧实际状态为准**（`state`/`is_enabled`），不信任store里的期望值——用户可在任务管理器/系统设置绕过软件改动；设置页在窗口重新获得焦点时刷新（消除与任务管理器的状态显示延迟）。托盘不提供自启动入口（见「托盘」第5条），状态显示与切换集中在设置页，`settings_get`返回`autoStartState`（enabled/disabled/disabled_by_user/disabled_by_policy/unsupported），unsupported时禁用开关
3. 静默启动依据：`--silent`参数（未打包），或`GetActivatedEventArgs().Kind == StartupTask`（打包版登录激活）
4. 设置页开关行不显示常驻状态提示；系统侧状态与开关不一致（开关开启但被用户/策略禁用）或环境不支持时，经toast提示一次（`notifyAutostartMismatch`，复用主页操作反馈形式）；`settings_get`返回`autoStartState`（enabled/disabled/disabled_by_user/disabled_by_policy/unsupported），unsupported时禁用开关
5. 打包版启用被系统拒绝（常见：用户曾在任务管理器/启动设置中禁用）时，设置页开关经command错误返回（`App.Status.StartupSettingDisabledByUser`等文案键），前端toast提示——不使用系统通知：用户可能关闭应用通知权限或开启专注助手导致提示不可见；未打包路径的注册表写入失败同样走此反馈（托盘侧自启动开关已移除，不再产生`autostart-error`事件，该事件通道已删除）

## 打开设置

托盘菜单「打开设置」：显示主窗口并导航到设置页。导航经双通道保证：

1. 窗口存活时后端emit `open-settings`事件即时切换
2. webview销毁重建场景下事件早于前端监听注册：后端置挂起标记（`OPEN_SETTINGS_REQUESTED`），前端挂载时经`take_open_settings_request`命令消费兜底；事件路径同样消费标记，防止陈旧标记在无关重开窗口时误导航
6. 用户从托盘菜单点击"打开主窗口"时，显示主窗口并聚焦
7. 主窗口默认隐藏创建，仅在需要时显示，使静默启动无需特殊分支

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
