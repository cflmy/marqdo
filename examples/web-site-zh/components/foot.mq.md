---
markup: occlusion
title: 公共页脚的终端 Markdown
import:
  组件: ext/web/网页/组件/索引.mq.md
  文件: lib/文件.mq.md
---

公共页脚只有项目与源码链接，内部没有独立双栏，因此简单 Markdown 就是拆分终点。父层决定它出现在何处，主题沿用原来页脚的颜色、间距和字号。

## foot

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | 保留原始链接的语义页脚 |

以站点为文件根授予 fs.read。内容叶子保留原项目与源码目的地，不查询业务记录，也不读取凭据。改文案只修改 content/footer.md；改几何回到父布局。组件输出仍是嵌套节点，不先把叶子渲染成不透明 HTML 再拼回页面。

我们 **文本 = [读取终端项目链接](文件.读文本(路径="components/content/footer.md"))** 和 **叶子 = [保留普通 Markdown 链接](组件.正文(文本=`文本`, 槽="footer-content"))**。
最后返回 *[保留页脚语义与主题类](组件.包裹(子节点=`叶子`, 种类="footer", 槽="footer", 样式="foot"))*。

