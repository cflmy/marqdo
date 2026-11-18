//! Reader-facing projection of compiler facts; no evaluation or invented evidence.
use crate::ast::{Function, Module};
use crate::sema::{SemanticArtifact, SemanticProject};
use crate::view::{html::escape, render::expr_display};

fn find_function(functions: &[Function], line: u32) -> Option<&Function> {
    functions.iter().find_map(|f| {
        if f.span.line == line {
            Some(f)
        } else {
            find_function(&f.children, line)
        }
    })
}
fn state(value: &str) -> String {
    match value {
        "passed" => "检查通过".into(),
        "failed" => "检查失败".into(),
        "unchecked" => "尚未检查".into(),
        "not_run" => "分析未运行程序".into(),
        "unknown" => "没有可用记录".into(),
        "sourced" => "已关联来源，内容尚未核验".into(),
        "verified" => "证据已核验".into(),
        "unverified" => "尚无已核验证据".into(),
        "unreviewed" => "尚未进行领域审阅".into(),
        other => format!("原始状态：{other}"),
    }
}
fn effect(value: &str) -> &str {
    match value {
        "pure" => "无已知宿主副作用",
        "fs.read" => "读取文件",
        "fs.write" => "写入文件",
        "net.http" | "network" => "网络请求",
        "process.exec" => "启动进程",
        "plugin" => "加载原生插件",
        "environment.read" => "读取环境配置",
        "output.write" => "输出文本",
        "input.read" => "读取输入",
        "unknown" => "调用效应尚不能确定",
        _ => value,
    }
}
fn kind(value: &str) -> &str {
    match value {
        "type" => "对象",
        "function" => "函数",
        "constant" => "常量",
        _ => value,
    }
}
fn step_kind(value: &str) -> &str {
    match value {
        "derive" => "计算",
        "observe" => "观察",
        "iterate" => "遍历",
        "call" => "调用",
        "return" => "返回",
        "branch" => "条件",
        "narrate" | "reference" => "正文引用",
        _ => value,
    }
}
fn render_steps(
    project: &SemanticProject,
    steps: &[&crate::sema::ir::KnowledgeStep],
    title: &str,
) -> String {
    if steps.is_empty() {
        return String::new();
    }
    let mut out=format!("<details class=\"knowledge-steps\"><summary>{}</summary><p class=\"knowledge-hint\">这是源码结构，不是实际执行时序；正文引用不会因阅读而执行。</p><ol class=\"knowledge-step-list\">",escape(title));
    for step in steps {
        let label = project
            .refs
            .iter()
            .find(|r| r.id == step.reference)
            .map(|r| r.label.as_str())
            .unwrap_or("未命名步骤");
        out.push_str(&format!(r##"<li class="knowledge-step" data-step-kind="{kind}" data-line="{line}"><span class="knowledge-step-kind">{meaning}</span><a href="#stmt-{line}">{label}</a></li>"##,kind=escape(&step.kind),line=step.source.line,meaning=escape(step_kind(&step.kind)),label=escape(label)));
    }
    out.push_str("</ol></details>");
    out
}
fn technical(artifact: &SemanticArtifact) -> String {
    let v = &artifact.verification;
    format!(
        r##"<details class="knowledge-technical"><summary>技术详情：身份、摘要与原始状态</summary><dl><dt>目标标识</dt><dd><code>{target}</code></dd><dt>制品摘要（未检查锁定状态）</dt><dd><code>{digest}</code></dd><dt>依赖标识</dt><dd><code>{deps}</code></dd><dt>原始验证状态</dt><dd><code>parse: {parse}; resolve: {resolve}; contract: {contract}; execution: {execution}; tests: {tests}; evidence: {evidence}; domain: {domain}</code></dd></dl></details>"##,
        target = escape(
            artifact
                .symbol_id
                .as_ref()
                .map(|id| id.as_str())
                .unwrap_or(&artifact.artifact_id.0)
        ),
        digest = escape(&artifact.hashes.artifact.0),
        deps = escape(
            &artifact
                .dependencies
                .iter()
                .map(|id| id.0.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        ),
        parse = escape(&v.parse),
        resolve = escape(&v.resolve),
        contract = escape(&v.contract),
        execution = escape(&v.execution),
        tests = escape(&v.tests),
        evidence = escape(&v.evidence),
        domain = escape(&v.domain)
    )
}
/// Facts are grouped by exact source unit, with machine identities behind disclosure.
pub(super) fn render(project: &SemanticProject, rel: &str, module: Option<&Module>) -> String {
    let mut artifacts: Vec<_> = project
        .artifacts
        .iter()
        .filter(|a| a.source.file == rel && a.kind != "module")
        .collect();
    artifacts.sort_by_key(|a| a.source.line);
    let steps: Vec<_> = project
        .steps
        .iter()
        .filter(|s| s.source.file == rel)
        .collect();
    if artifacts.is_empty() && steps.is_empty() {
        return String::new();
    }
    let mut out = format!(
        r##"<div class="knowledge-panel" aria-label="可执行知识"><div class="knowledge-intro"><h3>这份文档怎样成为可调用的知识</h3><p>{} 个知识单元 · {} 个结构步骤。先读输入、返回与调用边界，再按需查看依赖和依据。</p><p class="knowledge-hint">以下来自静态分析，不代表程序已经运行、测试通过或内容已获领域确认。实际运行见正文标注与“运行与结果”。</p></div>"##,
        artifacts.len(),
        steps.len()
    );
    let first = artifacts.first().map(|a| a.source.line).unwrap_or(u32::MAX);
    let introductory: Vec<_> = steps
        .iter()
        .copied()
        .filter(|s| s.source.line < first)
        .collect();
    out.push_str(&render_steps(project, &introductory, "正文中的引用与步骤"));
    for (index, a) in artifacts.iter().enumerate() {
        let line = a.source.line;
        let function = module.and_then(|m| find_function(&m.functions, line));
        let end = artifacts
            .get(index + 1)
            .map(|a| a.source.line)
            .unwrap_or(u32::MAX);
        out.push_str(&format!(r##"<details class="knowledge-card" id="knowledge-fn-{line}" data-source-line="{line}"><summary><span class="knowledge-unit-name">{name}</span><span class="knowledge-unit-kind">{kind}</span></summary><div class="knowledge-unit-body"><a class="knowledge-source-link" href="#fn-{line}">回到正文 · 第 {line} 行</a>"##,name=escape(&a.display_name),kind=escape(if function.is_some_and(|f|f.name=="main"&&f.level==1){"程序入口"}else{kind(&a.kind)})));
        if let Some(signature) = &a.signature {
            if signature.receiver == "instance" {
                out.push_str("<p class=\"knowledge-hint\">实例方法：先创建所属对象，再通过同一实例调用。</p>");
            }
            out.push_str("<h4>需要什么输入</h4>");
            if signature.params.is_empty() {
                out.push_str("<p>无需提供输入参数。</p>");
            } else {
                out.push_str("<div class=\"knowledge-table-wrap\"><table><thead><tr><th>参数</th><th>类型</th><th>提供方式</th><th>含义</th></tr></thead><tbody>");
                for param in &signature.params {
                    let authored = function
                        .and_then(|f| f.contract.as_ref())
                        .and_then(|c| c.params.iter().find(|p| p.name == param.name));
                    let default = function
                        .and_then(|f| f.params.iter().find(|p| p.name == param.name))
                        .and_then(|p| p.default.as_ref())
                        .map(expr_display);
                    let supply = if param.required {
                        "必填".into()
                    } else {
                        default
                            .map(|v| format!("默认 {v}"))
                            .unwrap_or("可省略；默认值见源码".into())
                    };
                    out.push_str(&format!(
                        "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        escape(&param.name),
                        escape(&param.ty),
                        escape(&supply),
                        escape(
                            authored
                                .map(|p| p.desc.as_str())
                                .filter(|s| !s.is_empty())
                                .unwrap_or("源码未提供参数说明")
                        )
                    ));
                }
                out.push_str("</tbody></table></div>");
            }
            let returns = function
                .and_then(|f| f.contract.as_ref())
                .and_then(|c| c.returns.as_ref());
            out.push_str(&format!(
                "<h4>返回什么</h4><p><code>{}</code> · {}</p>",
                escape(&signature.returns),
                escape(
                    returns
                        .map(|r| r.desc.as_str())
                        .filter(|s| !s.is_empty())
                        .unwrap_or("源码未提供返回值说明；请结合正文实现阅读。")
                )
            ));
        } else {
            out.push_str("<p>这是程序入口或对象声明。成员函数的输入输出在各自说明中列出。</p>");
            if let Some(returns) = function
                .and_then(|f| f.contract.as_ref())
                .and_then(|c| c.returns.as_ref())
            {
                out.push_str(&format!(
                    "<h4>源码中的返回契约</h4><p><code>{}</code> · {}</p>",
                    escape(&returns.ty.name()),
                    escape(&returns.desc)
                ));
            }
        }
        out.push_str(&format!(
            "<h4>调用边界</h4><p>{}。权限由实际运行入口授予；静态副作用信息不等于运行许可。</p>",
            escape(
                &a.effects
                    .values()
                    .iter()
                    .map(|s| effect(s))
                    .collect::<Vec<_>>()
                    .join("、")
            )
        ));
        if !a.dependencies.is_empty() {
            out.push_str("<h4>依赖哪些知识</h4><ul class=\"knowledge-dependencies\">");
            for dependency in &a.dependencies {
                if let Some(target) = project
                    .artifacts
                    .iter()
                    .find(|other| other.artifact_id == *dependency)
                {
                    if target.source.file == rel && target.kind != "module" {
                        out.push_str(&format!(
                            r##"<li><a href="#knowledge-fn-{}" data-knowledge-link>{}</a></li>"##,
                            target.source.line,
                            escape(&target.display_name)
                        ));
                    } else {
                        out.push_str(&format!(
                            "<li>{} <span class=\"knowledge-hint\">{}</span></li>",
                            escape(&target.display_name),
                            escape(&target.source.file)
                        ));
                    }
                } else {
                    out.push_str("<li>依赖尚未解析；完整标识见技术详情。</li>");
                }
            }
            out.push_str("</ul>");
        }
        let v = &a.verification;
        out.push_str(&format!("<h4>当前有哪些依据</h4><dl class=\"knowledge-status\"><dt>语法与引用</dt><dd>{}；{}</dd><dt>契约检查</dt><dd>{}</dd><dt>测试记录</dt><dd>{}</dd><dt>内容来源</dt><dd>{}</dd><dt>领域审阅</dt><dd>{}</dd></dl>",escape(&state(&v.parse)),escape(&state(&v.resolve)),escape(&state(&v.contract)),escape(&state(&v.tests)),escape(&state(&v.evidence)),escape(&state(&v.domain))));
        let owned: Vec<_> = steps
            .iter()
            .copied()
            .filter(|s| s.source.line >= line && s.source.line < end)
            .collect();
        out.push_str(&render_steps(project, &owned, "查看本单元的结构步骤"));
        out.push_str(&technical(a));
        out.push_str("</div></details>");
    }
    out.push_str("</div>");
    out
}
