# 电源计划

电源计划的读取、切换与卓越性能计划逻辑。返回 [DEVELOPMENT.md](../DEVELOPMENT.md)。

## 电源计划

1. 通过原生API实现以下能力：
   1. 枚举计划与读取名称：`PowerEnumerate` + `PowerReadFriendlyName`
   2. 获取当前计划：`PowerGetActiveScheme`
   3. 切换计划：`PowerSetActiveScheme`
   4. 复制计划：`PowerDuplicateScheme`
   5. 恢复默认计划：`PowerRestoreDefaultPowerSchemes`
2. 读取用户拥有的Windows电源计划
3. 检查用户是否有卓越性能计划，若无，则提供创建卓越性能计划选项
4. 实测注意：`PowerReadFriendlyName`首调（空缓冲）可能返回`SUCCESS`（size=所需字节数）或`MORE_DATA`，两者都要处理；部分内置计划（如平衡、节能模式）无可读名称，此时返回空串，UI层与托盘回退显示本地化默认名称（前端键`Main.DefaultPlanName`，后端键`tray-plan-default`）

## 创建卓越性能计划

1. 通过`PowerDuplicateScheme`复制系统卓越性能模板GUID：`e9a42b02-d5df-448d-aa00-03f14749eb61`（等价于`powercfg -duplicatescheme`）
2. 创建前先做存在性校验，避免重复创建留下多余副本
3. 创建后读取系统返回的UUID并保存，存储字段见[持久化设置](./settings.md)

## 卓越性能计划存在性

1. 部分设备通过`powercfg -l`无法查看到被隐藏的卓越性能计划
2. 针对这些设备，通过读取创建时的UUID，然后可开启卓越性能计划
3. 使用`PowerSetActiveScheme`传入创建时保存的UUID，可开启被隐藏的卓越性能计划
4. 但是用户通过其他途径删除该计划后，激活可能会失败，此时需提示用户
5. 在设置中恢复电源计划后，需清空储存的卓越性能计划的UUID

## 卓越性能计划识别逻辑

1. 仅将以下两类计划视为卓越性能计划：
   1. GUID等于系统卓越性能模板GUID：`e9a42b02-d5df-448d-aa00-03f14749eb61`
   2. GUID等于本程序保存的卓越性能计划UUID
2. 不通过计划名称关键词判断卓越性能计划，避免用户将普通计划改名后被误判
3. 用户已有但未被本程序记录的卓越性能副本仍会显示在计划列表中，用户可直接手动切换
4. 对于未被本程序记录的卓越性能副本，创建提示属于保守提示，用户可忽略
