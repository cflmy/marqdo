# View：动作留在正文，逻辑留在侧注

本轮针对阅读界面，不改变语言求值和工具权限。阅读示例为
[view-reading.mq.md](../../examples/view-reading.mq.md)。

遮蔽按真实 AST 识别，随后按源片段保留句子：方括号的动作说明留在正文，圆括号中的
实现进入独立侧注。同一句话可以包含多个调用和返回，不再把整行当成一次赋值。
文字字面量中的链接、普通 Markdown 导航与 classic 取元不冒充执行遮蔽。
空圆括号明确显示“待补充实际逻辑”，并说明当前空遮蔽返回 None。

宽屏实现侧注与运行标注共用现有的左右避碰及引出线布局，彼此有明确名称区分。
实现侧注标记“源码说明 · 不是运行结果”，不带调用帧或迭代帧身份。
密集区域、窄屏和无脚本页面使用可原生展开的 details；动作按钮可按需查看完整逻辑。
悬停与键盘聚焦关联实际源片段，不触发运行。

“怎样读这份可执行文档”沿用核心构造注册表，说明参数、赋值、返回、调用、条件、
循环、数据表、代码块、公式、写回与兼容标记。公式和外语代码仍使用既有卡片，
不新增语法或作者必填的展示配置。

可执行知识按真实源单元分组。默认先读输入、返回、调用边界、依赖和验证依据，
身份、摘要与原始状态收进技术详情。契约说明来自已解析的参数和返回表；缺失时如实说明。
语法检查、契约检查、测试、来源和领域审阅分别展示，静态分析不代表实际执行或内容正确。
源码结构步骤与实际运行时序分开；内存面板先展示最终绑定与调用记录，再按需展开原始报告。

函数的说明链接和知识卡片使用当前文档源行定位，避免同名方法共用一个 DOM 身份。
单文件阅读只分析它和实际导入的依赖，避免无关同目录模块重名导致知识区整体失败。
目录阅读保留项目分析。来源身份及版本匹配沿用现有编译器和页面 token，不另建求值器。

验收入口：

```sh
cargo test --offline --lib view::
cargo test --offline --test view_readability
cargo test --offline --test gold view_
cargo test --offline --test executable_knowledge
marqdo view output examples/view-reading.mq.md --no-exec --out /tmp/marqdo-reading
```

浏览器验收脚本是 [readability.browser.cjs](../../tests/view/readability.browser.cjs)，
接收真实页面 URL，支持 MARQDO_PLAYWRIGHT_MODULE、MARQDO_CHROMIUM 和 MARQDO_SCREENSHOT_DIR。
覆盖七种宽度、侧注边界与碰撞、多动作句子、空逻辑、知识卡片、正文定位和无脚本回退。
原有 [reader.browser.cjs](../../tests/view/reader.browser.cjs)继续覆盖曲线、帧选择及实时/静态交互。

本轮实际验收（2026-11-08）：67 项 view 相关单元测试、2 项新增只读导出测试通过；
全量回归 984 通过、0 失败、2 个原有忽略项。真实 Chromium 验证静态快照、
只读导出和实时页面，覆盖七种宽度、无脚本阅读、曲线与帧切换、源码定位及实时重新运行。
GOFLAGS 仅对完整回归命令关闭工作区所有权引起的 Go VCS 盖章失败，不改全局配置。
