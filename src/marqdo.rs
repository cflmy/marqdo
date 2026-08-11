//使用rust开发marqdo语言的解释器

//导入库
use std::env;   //用于获取命令行参数
use std::fs;    //用于读取文件
// use std::io; //用于读取标准输入和输出
// use std::io::BufReader; //用于读取缓冲区

//导入错误报告模块
mod error;
use error::ErrorReport;

fn main() {
    //创建新的错误报告器
    let mut error_report = ErrorReport::new();
    //获取命令行参数
    let args: Vec<String> = env::args().collect();
    
    //解析命令行参数
    if args.len() > 3 {
        eprintln!("Usage: marqdo <use_mode> <file>");
        std::process::exit(1);
    }
    else if args.len() == 3 {
        let use_mode = &args[1];
        let file_path = &args[2];
        //传入模式选择函数，根据不同的模式选择不同的函数执行
        mode_selector(use_mode, file_path, &mut error_report);
    }
    else if args.len() == 2 {
        let file_path = &args[1];
        //默认为run模式，自动进行文件执行
        run_file(file_path, &mut error_report);
    }
    else {
        //无参数输入，默认进入交互模式
        run_prompt(&mut error_report);
    }
}

//模式选择函数
fn mode_selector(use_mode: &str, file_path: &str, error_report: &mut ErrorReport) {
    //根据不同的模式选择不同的函数执行
    match use_mode {
        "run" => run_file(file_path, error_report),
        _ => {eprintln!("Invalid use mode: {}", use_mode);
            std::process::exit(2);
        }
    }
}

//读取marqdo文件并运行marqdo脚本文件
fn run_file(file_path: &str, error_report: &mut ErrorReport) {
    match fs::read_to_string(file_path) {
        Ok(source) => {
            run(&source, error_report);
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(3);
        }
    }
}

//交互式模式
fn run_prompt(error_report: &mut ErrorReport) {
    //提示交互式模式正在开发中
    println!("Interactive mode is not available yet.");
    error_report.error(0, "Interactive mode is not available yet.");
    //清除错误报告
    error_report.clear();
    std::process::exit(4);
}

//运行marqdo脚本
fn run(source: &str, error_report: &mut ErrorReport) {
    //提示正在开发中
    println!("Running script is not available yet.");
    //输出source
    println!("Source: {}", source);
    //输出错误报告
    println!("Error report: {}", error_report.has_error());
    //退出程序
    std::process::exit(5);
}