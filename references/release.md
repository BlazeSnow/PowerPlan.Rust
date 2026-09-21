# 发布

打包与发布流程。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

1. 软件使用GitHub action进行打包
2. 构建x64与arm64（`aarch64-pc-windows-msvc`）两个架构
3. 产物打包为msixbundle：配置AppxManifest后用`MakeAppx`打包；商店提交使用商店签名，本地测试需自签名证书
4. 图标资源（msix/assets）须包含scale/targetsize/altform-unplated全套变体，打包时经`MakePri`生成resources.pri——否则MRT无法解析变体，任务栏/开始菜单图标会被系统垫色底（Store上架实测问题）
5. 发布至Microsoft store
6. 版本号在`tauri.conf.json`中维护（以package.json为源头经version.ps1同步），与更新日志同步
