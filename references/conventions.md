# 通用约定

跨功能的通用开发约定。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

## 测试

1. 后端：`cargo test`（src-tauri目录）；业务核心（`core/`）不依赖Tauri运行时，可独立测试；后端i18n资源完整性（7语言ftl）与设置序列化契约（前端camelCase字段）均有回归测试
2. 前端：`pnpm test`（vitest + jsdom + @testing-library/react，`pnpm test:watch`监听模式）；纯逻辑放`src/lib`便于测试（如`plan.ts`的卓越卡片三态与复制预填名）；组件测试用`vi.mock`屏蔽Tauri invoke/event，断言文案经i18n资源解析避免硬编码语言

## 编码

1. 所有文件以UTF-8格式存储
2. Rust调用Windows原生API使用宽字符（UTF-16）接口，避免依赖系统ANSI代码页的窄字符接口
3. 中文Windows终端默认代码页为GBK(936)：调试输出、外部命令输出或脚本可能出现乱码，必要时以`chcp 65001`切换为UTF-8；程序逻辑不得依赖终端代码页

## 性能要求

1. 程序主要运行在主界面关闭、托盘开启的状态，因此需要着重优化该状态时的性能与系统占用
2. 托盘常驻全程零webview：主窗口关闭即销毁webview，启动到托盘/静默启动时根本不创建（`create: false`延迟创建），打开时按需重建；webview进程组（内存大头）仅在主窗口打开期间存在
3. 托盘常驻状态下无定时轮询（事件驱动刷新，计划列表带缓存）；唯一周期性工作为系统主题监听每秒一次的注册表读取
