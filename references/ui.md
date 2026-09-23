# 界面

主页面、侧边栏与标题栏约定。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

界面使用shadcn/ui自身的设计风格，不仿制Windows系统应用。

## 主页面

1. 主页包括：卓越性能计划卡片、电源计划列表、电源选项入口
2. 卓越性能计划卡片三态（对齐旧版行为）：
   1. 已有卓越性能计划（模板GUID或储存UUID出现在计划列表中）：隐藏卡片
   2. 储存UUID存在但计划不在列表中（计划被隐藏）：显示激活按钮；激活失败说明计划已被删除，清空储存UUID并提示
   3. 完全没有：显示创建按钮
3. 电源计划列表：
   1. 展示用户的Windows电源计划
   2. 通过单选切换当前使用的计划
   3. 切换成功的提示显示计划名称，不显示GUID
   4. 每行提供复制按钮：对话框预填「名称 - 副本」（空名回退默认名称），确认后复制并重命名
   5. 列表标题提供刷新按钮，跳过缓存强制刷新
4. 电源选项入口：打开控制面板的电源选项（`control /name Microsoft.PowerOptions`）
5. 操作结果提示统一由Toast承担（对齐旧版状态栏职能），不设独立状态栏；旧版`Main.Status.*`键继续用于Toast文案

## 侧边栏

1. 使用shadcn/ui侧边栏实现
2. 切换主页和设置页
3. 伸缩侧边栏按钮与页面标题同行，位于内容区顶部（标题栏已交还系统，不设独立顶栏）

## 标题栏

1. 使用系统标题栏（窗口默认decorations），不自绘拖拽区与窗口控制按钮
2. 窗口标题为产品名（tauri.conf.json的title）

## 窗口持久化

1. 使用`tauri-plugin-window-state`保存并恢复用户手动调整的窗口大小、位置与最大化/全屏状态
2. 状态标志必须排除VISIBLE：窗口默认隐藏创建（静默启动），不能恢复出可见状态
3. 启动时钳制到当前显示器工作区：尺寸超限收缩、位置越界回位；首次启动（无状态文件）在工作区居中

## 深色模式

1. 启动首帧：index.html头部内联脚本按prefers-color-scheme预设dark类与深色背景（#0a0a0a，与.dark主题--background一致），消除深色模式启动白屏；prefers-color-scheme仅启动时可靠
2. 运行中：WebView2的prefers-color-scheme不保证随系统实时更新，由后端监听系统设置变更广播（WM_SETTINGCHANGE，隐藏顶层窗口接收——message-only窗口收不到广播，不能用），广播到来时核对注册表（AppsUseLightTheme），变化时设置窗口原生主题并emit `system-theme`事件；直接写注册表不广播的场景收不到（正规主题切换均广播）；`system_theme`命令供前端挂载时兜底查询
3. 前端`applySystemTheme`切换文档dark类并写内联color-scheme（覆盖wry建窗时按"系统模式"写入的内联值）；挂载后延迟一拍应用初始值，避免被next-themes挂载效果覆盖
4. Toast（sonner）的theme由同一主题状态驱动，跟随系统深浅色；sonner自带的"system"模式走prefers-color-scheme，与窗口同样不实时，不得使用
