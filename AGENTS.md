# PowerPlan 开发指南

1. 禁止修改本文件
2. 开发过程中需要处理终端GBK与UTF-8的关系
3. 更新完一项功能后，修改CHANGELOG.md和DEVELOPMENT.md

## 软件架构

1. 软件使用tauri2架构
2. 软件后端使用rust
3. 软件UI使用shadcn/ui

## 软件逻辑

1. 通过Windows原生API读取与切换电源计划

## 发布

1. 软件使用GitHub action进行打包
2. 打包x64和arm64架构为msixbundle
3. 发布至Microsoft store
