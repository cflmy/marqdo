//解释器检测到错误的处理模块

//! 错误报告模块，集中管理解释器所有语法错误输出
// use std::io;
// use std::io::Write;

//建立独立抽象ErrorReport结构体
//错误报告器，保存错误状态并提供错误报告接口
#[derive(Debug, Default)]
pub struct ErrorReport {
    had_error: bool,
}

impl ErrorReport {
    //创建新的错误报告器
    pub fn new() -> Self {
        Self { had_error: false }
    }

    //对外公开的错误接口，指定行号和错误信息
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    //内部私有函数，输出错误信息，标记错误状态为true
    fn report(&mut self, line: usize, location: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, location, message);
        self.had_error = true;
    }

    //获取错误状态
    pub fn has_error(&self) -> bool {
        self.had_error
    }

    //重置错误状态
    pub fn clear(&mut self) {
        self.had_error = false;
    }
}