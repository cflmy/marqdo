---
title: Marqdo主分支开发记录
date: 2026-08-11
author: pingan
---

## 前言
在经过一周多的ai疯狂开发之后，尽管marqdo已经做出了并拥有了许多非常良好的特性，作者确逐渐感到非常的不安，
这种不安来自于崩塌感，作者不知道什么时候，ai就会更改关键的定义，导致突然的崩塌，在开发智能体拓展库的时候作者还尚未有这种感觉，但是在网络拓展库，这种不安定感几乎席卷了作者的全部身心。

必须承认，ai在没有经过大量训练就撰写一门崭新的语法，定义一些它完全没有接触过的东西对于ai还是太困难了，ai并不能通过我的监督，真正的理解marqdo代码即文档的设计思路，所以，尽管非常的遗憾，作者还是决定回归古法编程，还是手搓代码吧，尽管这意味着marqdo很难赶上ai的浪潮了。

marqdo的设计初衷很是简单，这就不是一门设计给程序员的语言，哪个程序员会用markdown表格来表示集合呢？不会的，这太过繁复了，但是ai可以，这就使得过去编程语言的范式变得不是那么重要了。

自然如果只是为了易读性，marqdo并没有完全的开发必要，重点在于对md的支持，作者必须承认谷歌提出的OKF规范，把markdown这种文件格式放到了一个极高的位置，这意味着，可执行的知识体系，如果能做到，为什么不试试？

此外，jupyter notebook笔记本体积太大不利于管理的问题也是存在的，作者就在想，markdown支持代码块的插入，这个代码如果可以直接执行并放回，这不就是最好的jupyter替代品吗？事实上，在main分支的marqdo已经实现了这个功能，但是还不够好，作者要把marqdo真正做成一门科学计算的顶级语言。

最后是数学公式的支持，公式库需要能够自动的渲染，高可读，还要能够自动的画图，自动的完成诸多内容，让可执行=正确，让ai支持数学推理不再困难。

最后，2026年8月11日，我预祝未来的自己，开发成功。

## 开发

### Marqdo上下文无关文法

file -> "---" metadata "---" content ;

metadata    ->  file_information |
                import_lib_name ;

content     ->  comment |
                code_body ;

comment     ->  comment_content ;

code_body   ->  class |
                function|
                code|
                $formula$|$$formula$$|\[formula\]|
                ```code_block_type 
                    code_block
                ```;

class       ->  # class_name code_body |
                # class_name => parent_class_name code_body ;

function    -> function_header param_list body return_stmt ;

function_header     ->  ## identifier | 
                        ### identifier | 
                        #### identifier ;

param_list  ->  + identifier ;

body        ->  statement | 
                branch | 
                loop ;

return_stmt -> ** statement ** ;

<!--the number of # stand for the level of function -->

code        ->  1. branch |
                - loop |
                statement ;

<!--Ordered lists represent branch, unordered lists represent loop-->

branch      -> statement ;

loop        -> statement ;

statement   ->  > call |
                literal|
                `var_value_name`|
                list and dict|
                "str"|
                *statement_without_any_marker*;

<!--here we use markdown table to stand for list and dict-->

call        -> function_name args;

literal     -> literal_code as other code;

formula     -> latex math code;

code_block_type -> code_block's code type;

code_block  -> code of a code_block;