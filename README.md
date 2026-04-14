

# Tavi C Compiler by Rust



## Environment

* Mac Silicon (M4)
* Docker
* Rust


## Dependencies

* Docker
* Docker Compose
* [cross](https://crates.io/crates/cross)
* [cargo-make](https://crates.io/crates/cargo-make)


## Reference

* [低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)


## Run

実行方法など

```bash
# ビルドだけ
$ cargo make build

# ビルドとテスト
$ cargo make flow

# Docker内に入ってデバッグ
$ cargo make login
# Docker内
$ ./gdb/mygdb.sh ./target/tmp/tmp
```


デバッグでよく使う命令

```gdb
# main関数にブレークポイント (デバッグ開始したときはほぼこれ)
$ b main

# ブレークポイントまで進める
$ c

# 1行実行
$ si

# main関数内の特定アドレスにブレークポイント (下記では+48)
$ b *(main+48)

# アセンブリ見たいとき
$ disass

# 終了
$ exit
```



## EBNF

Extended BNF for TVCC (Recursive Descent Parsing)

```
program       = func_def*
func_def      = type ident "(" func_args ")" "{" compound_stmt
func_args     = (type ident ("," type ident )*)?
stmt          = "return" expr ";"
                | "if" "(" expr ")" stmt ("else" stmt)?
                | "while" "(" expr ")" stmt
                | "for" "(" expr? ";" expr? ";" expr? ")" stmt
                | "{" compound_stmt
                | expr? ";"
compound_stmt = (declaration | stmt)* "}"
declaration   = declspec (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"
declspec      = type
declarator    = ident
expr          = assign
assign        = equality ("=" assign)?
equality      = relational ("==" relational | "!=" relational)*
relational    = add ("<" add | "<=" add | ">" add | ">=" add)*
add           = mul ("+" mul | "-" mul)*
mul           = unary ("*" unary | "/" unary)*
unary         = ("+" | "-" | "&" | "*")? primary
primary       = "(" expr ")"
                | ident ("(" fcall_args ")")?
                | num 
type          = "int"
fcall_args    = (ident ("," ident)*)?
```


## Progress

- [x] ステップ1: 整数1個をコンパイルする言語の作成
- [x] ステップ2: 加減算のできるコンパイラの作成
- [x] ステップ3: トークナイザを導入
- [x] ステップ4: エラーメッセージを改良
- [x] ステップ5: 四則演算のできる言語の作成
- [x] ステップ6: 単項プラスと単項マイナス
- [x] ステップ7: 比較演算子
- [x] ステップ8: ファイル分割とMakefileの変更
- [x] ステップ9: 1文字のローカル変数
- [x] ステップ10: 複数文字のローカル変数
- [x] ステップ11: return文
- [x] ステップ12: 制御構文を足す
- [x] ステップ13: ブロック
- [x] ステップ14: 関数の呼び出しに対応する
- [x] ステップ15: 関数の定義に対応する
- [x] ステップ16: 単項`&`と単項`*`
- [x] ステップ17: intキーワードを導入
- [x] 番外: 複数変数宣言および初期化を同時にできるように。
- [x] 番外: Rustのエラーハンドリングをもっと最適化
- [ ] ステップ18: ポインタ型を導入
- [ ] ステップ19: ポインタの加算と減算を実装
- [ ] ステップ20: sizeof演算子
- [ ] ステップ21: 配列を実装
- [ ] ステップ22: 配列の添字を実装
- [ ] ステップ23: グローバル変数を実装
- [ ] ステップ24: 文字型を実装
- [ ] ステップ25: 文字列リテラルを実装
- [ ] ステップ26: 入力をファイルから読む
- [ ] ステップ27: 行コメントとブロックコメント
- [ ] ステップ28: テストをCで書き直す
