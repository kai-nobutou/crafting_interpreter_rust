use crate::lox::error::LoxError;
use lox::evaluator::EvalResult;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod lox;

/// エントリーポイント関数。
///
/// 引数が指定されていればスクリプトファイルを実行し、
/// 指定されていなければ対話型プロンプトを起動します。
fn main() -> Result<(), LoxError> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, script] => run_file(script)?,
        [_] => run_prompt()?,
        _ => {
            std::process::exit(64);
        }
    }
    Ok(())
}

/// 指定されたスクリプトファイルを実行します。
///
/// # 引数
/// - `path`: 実行するスクリプトファイルのパス。
///
/// # エラー
/// ファイルが見つからない場合、または実行中にエラーが発生した場合に `LoxError` を返します。
fn run_file(path: &str) -> Result<(), LoxError> {
    let source = fs::read_to_string(path).map_err(|_| LoxError::FileNotFound(path.to_string()))?;

    // 実行して結果を出力
    match run(&source) {
        Ok(output) => {
            println!("{}", output); // 成功時に出力を表示
            Ok(())
        }
        Err(err) => {
            eprintln!("Error: {}", err); // エラーを表示
            Err(err)
        }
    }
}

/// 対話型プロンプトを起動します。
///
/// ユーザーが入力したコードを1行ずつ評価します。
/// 各行のコードは保持された`Evaluator`インスタンスによって評価されるため、変数やスコープの状態が維持されます。
///
/// # 戻り値
/// - `Ok(())`: プロンプトが正常に終了した場合。
/// - `Err(LoxError)`: 入力または出力処理中にエラーが発生した場合。
///
/// # 挙動
/// 入力された各行が`run_with_evaluator`関数によって解析・評価され、
/// 結果が標準出力に表示されます。エラーが発生した場合、エラーメッセージが標準エラー出力に表示されます。
///
/// # 使用例
/// ```
/// // 実行時にプロンプトが起動します。
/// // > var x = 10;       // 変数の定義
/// // > print x;          // 変数の表示: 10
/// // >                  // 空行で終了
/// ```
fn run_prompt() -> Result<(), LoxError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut evaluator = lox::evaluator::Evaluator::new(); // プロンプト全体でEvaluatorを保持
    a();
    loop {
        write!(stdout, "> ")
            .map_err(|_| LoxError::IoError("Failed to write prompt".to_string()))?;
        stdout
            .flush()
            .map_err(|_| LoxError::IoError("Failed to flush stdout".to_string()))?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() || line.trim().is_empty() {
            break;
        }

        match run_with_evaluator(&line, &mut evaluator) {
            Ok(output) => println!("{}", output),
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    Ok(())
}

/// 指定されたソースコードを指定された`Evaluator`で評価します。
///
/// スキャナー、パーサーを順に呼び出し、構文解析を行った後、`Evaluator`によってコードを実行します。
/// `Evaluator`は呼び出し元から提供されるため、スコープや変数の状態が維持されます。
///
/// # 引数
/// - `source`: 評価するソースコード文字列。
/// - `evaluator`: ソースコードの評価を行う`Evaluator`インスタンス。
///
/// # 戻り値
/// - `Ok(String)`: 評価が成功した場合、実行結果を文字列で返します。
/// - `Err(LoxError)`: スキャナー、パーサー、または評価中にエラーが発生した場合。
///
/// # 使用例
/// ```
/// let mut evaluator = Evaluator::new();
/// let source = "var x = 10; print x;";
/// match run_with_evaluator(source, &mut evaluator) {
///     Ok(output) => println!("Result: {}", output), // Result: 10
///     Err(err) => eprintln!("Error: {}", err),
/// }
/// ```
fn run_with_evaluator(
    source: &str,
    evaluator: &mut lox::evaluator::Evaluator,
) -> Result<String, LoxError> {
    let mut scanner = lox::scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    if tokens.is_empty() {
        return Err(LoxError::ParseError("No tokens found".to_string()));
    }

    let mut parser = lox::parser::Parser::new(tokens);
    let statements = parser.parse()?;

    if statements.is_empty() {
        return Err(LoxError::ParseError(
            "No valid statements produced".to_string(),
        ));
    }

    match evaluator.evaluate_statements(statements) {
        EvalResult::Return(_) => Ok(evaluator.get_output()),
        EvalResult::Error(err) => Err(err),
    }
}

/// 指定されたソースコードを解析し、実行します。
///
/// # 引数
/// - `source`: 実行するソースコード。
///
/// # エラー
/// トークン化、パース、評価のいずれかでエラーが発生した場合に `LoxError` を返します。
fn run(source: &str) -> Result<String, LoxError> {
    let mut scanner = lox::scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    if tokens.is_empty() {
        return Err(LoxError::ParseError("No tokens found".to_string()));
    }

    let mut parser = lox::parser::Parser::new(tokens);
    let statements = parser.parse()?;

    if statements.is_empty() {
        return Err(LoxError::ParseError(
            "No valid statements produced".to_string(),
        ));
    }

    let mut evaluator = lox::evaluator::Evaluator::new();

    // 評価結果を取得
    match evaluator.evaluate_statements(statements) {
        EvalResult::Return(_) => Ok(evaluator.get_output()),
        EvalResult::Error(err) => Err(err),
    }
}


fn a() {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    // 青い"N"と"O"の部分
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Blue)))
        .unwrap();
    writeln!(
        &mut stdout,
        "  ███╗   ██╗ ██████╗ ██████╗ ██╗   ██╗████████╗ █████╗ ██╗"
    )
    .unwrap();
    writeln!(
        &mut stdout,
        "  ████╗  ██║██╔═══██╗██╔══██╗██║   ██║╚══██╔══╝██╔══██╗██║"
    )
    .unwrap();

    // 緑の"B"と"U"の部分
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
        .unwrap();
    writeln!(
        &mut stdout,
        "  ██╔██╗ ██║██║   ██║██████╔╝██║   ██║   ██║   ███████║██║"
    )
    .unwrap();
    writeln!(
        &mut stdout,
        "  ██║╚██╗██║██║   ██║██╔═══╝ ██║   ██║   ██║   ██╔══██║██║"
    )
    .unwrap();

    // 赤い"TOKAI"の部分
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .unwrap();
    writeln!(
        &mut stdout,
        "  ██║ ╚████║╚██████╔╝██║     ╚██████╔╝   ██║   ██║  ██║██║"
    )
    .unwrap();
    writeln!(
        &mut stdout,
        "  ╚═╝  ╚═══╝ ╚═════╝ ╚═╝      ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝"
    )
    .unwrap();

    // リセットしてウェルカムメッセージ
    stdout.reset().unwrap();
    writeln!(&mut stdout, "Welcome to the NOBUTOKAI Interpreter!").unwrap();
    writeln!(&mut stdout, "Type your commands below to start...").unwrap();
}
