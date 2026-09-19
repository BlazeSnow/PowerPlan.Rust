# 开发指南

PowerPlan 的详细开发指南。基本约定与内容速览见 [AGENTS.md](./AGENTS.md)。本文档允许修改，应随开发进度同步更新；每完成一项功能，同步更新 [CHANGELOG.md](./CHANGELOG.md) 与本文档。

## 程序目的

快速切换Windows电源计划

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

## 权限

1. 程序以普通用户权限运行，全程不申请管理员权限
2. 电源计划的读取、切换和创建卓越性能计划副本均通过`powrprof.dll`原生API实现，不需要管理员权限
3. 例外：设置页的恢复电源计划调用`PowerRestoreDefaultPowerSchemes`，按微软文档要求调用者必须是本地Administrators组成员；权限不足时操作失败，弹窗提示错误

## 电源计划

1. 通过原生API实现以下能力：
   1. 枚举计划与读取名称：`PowerEnumerate` + `PowerReadFriendlyName`
   2. 获取当前计划：`PowerGetActiveScheme`
   3. 切换计划：`PowerSetActiveScheme`
   4. 复制计划：`PowerDuplicateScheme`
   5. 恢复默认计划：`PowerRestoreDefaultPowerSchemes`
2. 读取用户拥有的Windows电源计划
3. 检查用户是否有卓越性能计划，若无，则提供创建卓越性能计划选项

### 创建卓越性能计划

1. 通过`PowerDuplicateScheme`复制系统卓越性能模板GUID：`e9a42b02-d5df-448d-aa00-03f14749eb61`（等价于`powercfg -duplicatescheme`）
2. 创建前先做存在性校验，避免重复创建留下多余副本
3. 创建后读取系统返回的UUID并保存

### 卓越性能计划存在性

1. 部分设备通过`powercfg -l`无法查看到被隐藏的卓越性能计划
2. 针对这些设备，通过读取创建时的UUID，然后可开启卓越性能计划
3. 使用`PowerSetActiveScheme`传入创建时保存的UUID，可开启被隐藏的卓越性能计划
4. 但是用户通过其他途径删除该计划后，激活可能会失败，此时需提示用户
5. 在设置中恢复电源计划后，需清空储存的卓越性能计划的UUID

### 卓越性能计划识别逻辑

1. 仅将以下两类计划视为卓越性能计划：
   1. GUID等于系统卓越性能模板GUID：`e9a42b02-d5df-448d-aa00-03f14749eb61`
   2. GUID等于本程序保存的卓越性能计划UUID
2. 不通过计划名称关键词判断卓越性能计划，避免用户将普通计划改名后被误判
3. 用户已有但未被本程序记录的卓越性能副本仍会显示在计划列表中，用户可直接手动切换
4. 对于未被本程序记录的卓越性能副本，创建提示属于保守提示，用户可忽略

## 主页面

1. 主页包括：卓越性能计划卡片、电源计划列表、电源选项入口、状态
2. 卓越性能计划卡片：
   1. 有一行卓越性能计划，提供开启按钮（若系统内无卓越性能计划）
3. 电源计划列表：
   1. 展示用户的Windows电源计划
   2. 通过单选切换当前使用的计划
   3. 切换成功的提示显示计划名称，不显示GUID
4. 电源选项入口：打开控制面板的电源选项（`control /name Microsoft.PowerOptions`）
5. 状态：
   1. 显示当前计划名称与当前时间，使用shadcn/ui组件展示
   2. 定时器仅在窗口可见时运行，窗口隐藏时暂停

## 托盘

1. 使用Tauri 2内置托盘（tray-icon）实现，图标使用应用图标（`src-tauri/icons`下的`.ico`）
2. 托盘菜单内容每次打开由当前快照生成：电源计划列表（当前计划使用勾选标识）、开机自启动开关、打开主窗口、退出
3. 托盘菜单深浅色跟随系统应用主题；若菜单库不支持，需通过系统兼容层实现；禁止`ForceDark`、`ForceLight`或硬编码菜单颜色
4. 关闭主窗口时若托盘启用：拦截关闭事件，隐藏窗口，应用保活
5. 托盘与主页面的计划状态保持同步：托盘切换计划后通知前端；主窗口隐藏期间的状态在下一次显示时刷新
6. 退出：先销毁托盘与菜单资源，再退出应用
7. Explorer重启后托盘图标需恢复（tray-icon库已内置`TaskbarCreated`处理，升级依赖后需回归验证）
8. 修改托盘相关实现后，必须验证：动态菜单刷新、系统浅深主题、静默启动、重复打开菜单、Explorer重启恢复和退出流程

### 开机自启动

1. 使用`tauri-plugin-autostart`实现（写入注册表HKCU Run项）
2. 自启动时附带静默参数；启动时解析启动参数，若带静默参数且托盘启用，主窗口不显示，直接进入托盘
3. 用户从托盘菜单点击"打开主窗口"时，显示主窗口并聚焦
4. 主窗口默认隐藏创建，仅在需要时显示，使静默启动无需特殊分支

### 开机自启动本地测试

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

## 设置页面

1. 设置界面仿制Windows 11设置应用（shadcn/ui实现）
2. 语言（下拉框）：切换显示语言，更改后提示重启应用
3. 开机自启动（开关）：默认为关
4. 启用托盘（开关）：默认为开
5. 启动到托盘（开关）：默认为关，需先启用托盘，启动时直接进入托盘不显示主窗口
6. 恢复电源计划（按钮）：恢复电源计划到默认状态`PowerRestoreDefaultPowerSchemes`，成功后清空储存的卓越性能计划UUID
7. 开发者官网（按钮）：<https://www.blazesnow.com>
8. 代码仓库（按钮）：<https://github.com/BlazeSnow/PowerPlan.Rust>
9. 软件版本号：显示当前安装包版本，从Tauri应用配置读取，不硬编码版本号

### 持久化设置

1. 设置内容通过`tauri-plugin-store`保存为JSON文件（应用数据目录）
2. 旧WinUI版（LocalSettings）的数据不迁移：两者包身份不同，无法跨包读取；卓越性能UUID缺失时按识别与创建逻辑重新处理
3. 存储字段：
   1. 语言：`language`
   2. 开机自启动：`auto_start_enabled`
   3. 启用托盘：`tray_enabled`
   4. 启动到托盘：`launch_to_tray`
   5. 卓越性能计划UUID：`ultimate_performance_plan_guid`

## 侧边栏

1. 使用shadcn/ui侧边栏实现，风格贴近WinUI原生侧边栏
2. 切换主页和设置页
3. 伸缩侧边栏按钮放在标题栏上

## 标题栏

1. 隐藏系统标题栏（窗口`decorations: false`），自绘标题栏：拖拽区域使用`data-tauri-drag-region`，自绘最小化、最大化、关闭按钮
2. 标题栏显示软件名称和软件简介
3. 软件简介同时作为标题栏副标题
4. 标题栏文本使用前端i18n资源，不硬编码可见字符串

## 语言

1. 使用前端i18n方案（如react-i18next）
2. 默认跟随系统语言，未匹配到受支持语言时回退为英语
3. 支持简体中文、繁体中文、英语、法语、意大利语、德语、西班牙语
4. Rust侧可见文本（托盘菜单等）与前端使用同一语言设置，语言更改后提示重启应用以完全生效

## 编码

1. 所有文件以UTF-8格式存储
2. Rust调用Windows原生API使用宽字符（UTF-16）接口，避免依赖系统ANSI代码页的窄字符接口
3. 中文Windows终端默认代码页为GBK(936)：调试输出、外部命令输出或脚本可能出现乱码，必要时以`chcp 65001`切换为UTF-8；程序逻辑不得依赖终端代码页

## 性能要求

1. 程序主要运行在主界面关闭、托盘开启的状态，因此需要着重优化该状态时的性能与系统占用
2. 主窗口隐藏时暂停定时器与计划刷新，显示时刷新一次；托盘常驻状态下事件驱动刷新，不做定时轮询

## 发布

1. 软件使用GitHub action进行打包
2. 构建x64与arm64（`aarch64-pc-windows-msvc`）两个架构
3. 产物打包为msixbundle：配置AppxManifest后用`MakeAppx`打包；商店提交使用商店签名，本地测试需自签名证书
4. 发布至Microsoft store
5. 版本号在`tauri.conf.json`中维护，与[CHANGELOG.md](./CHANGELOG.md)同步
