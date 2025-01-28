# Crafting Interpreter in Rust

このリポジトリは、[Crafting Interpreters](https://craftinginterpreters.com/) に記載されているインタプリタを Rust で実装したプロジェクトです。このプロジェクトの目標は、書籍に記載された Lox プログラミング言語のサブセットを実行するバイトコード仮想マシンを構築することです。

## 主な特徴

- **バイトコード仮想マシン**: スタックベースの仮想マシンを実装。
- **Lox 言語のサブセット**: クラスやメソッドを除いた基本的な機能に焦点を当てています。
- **Rust 実装**: Rust の特徴を活かした安全かつ効率的な実装。
- **拡張性**: 将来的に機能を追加しやすい構造。

## 始め方

### 必要なもの

- Rust（最新の安定版を推奨）
- `cargo`（Rust に付属しています）

### インストール

1. このリポジトリをクローンします:

   ```bash
   git clone https://github.com/kai-nobutou/crafting_interpreter_rust.git
   cd crafting_interpreter_rust
   ```

2. プロジェクトをビルドします:

   ```bash
   cargo build
   ```

3. テストを実行して正しく動作するか確認します:

   ```bash
   cargo test
   ```

## 実行方法

インタプリタを実行するには、以下のコマンドを使用します:

```bash
cargo run -- <loxスクリプトファイル>
```

`<loxスクリプトファイル>` には実行したい Lox スクリプトのパスを指定してください。

また、REPL（対話モード）を開始するには以下を実行します:

```bash
cargo run
```

## 実行例

以下の Lox コードを `example.lox` という名前で保存します:

```lox
print "Hello, Lox!";
```

インタプリタを以下のコマンドで実行します:

```bash
cargo run -- example.lox
```

出力例:

```
Hello, Lox!
```

## ライセンス

このプロジェクトは MIT ライセンスのもとで公開されています。詳細は [LICENSE](./LICENSE) ファイルを参照してください。
