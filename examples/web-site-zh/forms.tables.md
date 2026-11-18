# 文章表单数据

可调用契约在 forms.mq.md。这里保存原始字面行，不是另一份提交实现。字段对应既有 articles schema，原生展示和服务器校验使用同一记录。

## fields

标题与短链必填，摘要和正文可选，输入类型保持原样。

| @ | name | label | type | required |
|---|---|---|---|---|
| 1 | title | 标题 | text | True |
| 2 | slug | slug | text | True |
| 3 | summary | 摘要 | textarea | False |
| 4 | body | 正文 | textarea | False |

## rules

服务器使用这些原始规则与错误文案，浏览器提示不能替代它们。

| @ | field | rule | message |
|---|---|---|---|
| 1 | title | required | 标题不能为空 |
| 2 | title | max:120 | 标题太长 |
| 3 | slug | required | slug 不能为空 |
| 4 | slug | max:80 | slug 太长 |
| 5 | summary | max:500 | 摘要太长 |
| 6 | body | max:8000 | 正文太长 |
