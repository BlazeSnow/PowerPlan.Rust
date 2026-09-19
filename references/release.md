# 发布

打包与发布流程。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

1. 软件使用GitHub action进行打包
2. 构建x64与arm64（`aarch64-pc-windows-msvc`）两个架构
3. 产物打包为msixbundle：配置AppxManifest后用`MakeAppx`打包；商店提交使用商店签名，本地测试需自签名证书
4. 发布至Microsoft store
5. 版本号在`tauri.conf.json`中维护，与[CHANGELOG.md](../CHANGELOG.md)同步
