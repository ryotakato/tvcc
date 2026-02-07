# GDB Python コンテキスト表示スクリプト

## Context

tvcc（自作Cコンパイラ）のデバッグにおいて、GDB上で `si` → `i r` → `disass` を何百回も繰り返す苦痛を解消する。gef/pwndbg/pedaはRosettaのリモートデバッグスタブでは動作しないため、基本的なGDB機能のみを使う軽量なPythonスクリプトを自作する。

ステップ実行のたびに、レジスタ・スタック・ディスアセンブリを自動表示する。

## ファイル構成

| ファイル | 説明 |
|---------|------|
| `gdb/tvcc-ctx.py` | GDB Pythonスクリプト本体 |
| `gdb/mygdb.sh` | GDB起動スクリプト（Rosettaリモートデバッグ用） |

## コマンド

| コマンド | 機能 |
|---------|------|
| `ctx` | レジスタ + スタック + ディスアセンブリを手動表示 |
| `stack [n]` | スタックのみ表示（デフォルト16エントリ） |
| `ctx-on` | 自動表示を有効化 |
| `ctx-off` | 自動表示を無効化 |

## 自動表示

- `gdb.events.stop.connect(handler)` で停止イベントをフック
- `si`/`ni`/ブレークポイント停止のたびにコンテキストを自動表示
- デフォルトON、`ctx-off`でOFF

## レジスタ表示

- 対象: `rax`, `rdi`, `rsi`, `rdx`, `rcx`, `rbx`, `rbp`, `rsp`, `rip`
- `gdb.parse_and_eval("$reg")` で取得（リモートスタブ互換性が最も高い）
- 前回値からの変更を赤色でハイライト
- 2列レイアウトで表示をコンパクトに

## スタック表示

- `gdb.execute("x/16gx $rsp", to_string=True)` で取得
- RSP、RBPの位置をマーカーで表示
- 小さい値には10進数も併記（tvccのテストケースは小さい整数が多い）

## ディスアセンブリ表示

- まず `disassemble` を試み、失敗時は `x/7i $pc` にフォールバック
- 現在の命令を `=>` マーカーと色で強調

## Rosetta互換性の方針

- `gdb.parse_and_eval("$reg")` — レジスタ読み取り（基本機能）
- `gdb.execute("x/...", to_string=True)` — メモリ読み取り（最も低レベルなアクセス）
- 全操作を try/except で包み、失敗時は `<cannot read>` を表示（クラッシュしない）

## 使い方

```bash
# テストプログラムをコンパイル・デバッグ
echo 'main() { a=3; b=5; return a+b; }' | ./target/x86_64-unknown-linux-musl/debug/tvcc > /tvcc/target/tmp/dbg_test.s
gcc -o /tvcc/target/tmp/dbg_test /tvcc/target/tmp/dbg_test.s
/tvcc/gdb/mygdb.sh /tvcc/target/tmp/dbg_test
```

GDB内:
```
b main
c
si          # コンテキストが自動表示される
stack 32    # 32エントリのスタック表示
ctx-off     # 自動表示OFF
ctx-on      # 自動表示ON
ctx         # 手動でコンテキスト表示
```

## 出力例

```
------------------------------------------------------------
[ Registers ]
  rax : 0x0000000000000003 | rdi : 0x0000000000000005
  rcx : 0x0000000000000000 | rsi : 0x0000000000000000
  rdx : 0x0000000000000000 | rbx : 0x0000000000000000
  rbp : 0x00007fffffffe0a0 | rsp : 0x00007fffffffe090
  rip : 0x0000000000401028
------------------------------------------------------------
[ Stack (from RSP) ]
  0x7fffffffe090 : 0x0000000000000003  (3)     <-- RSP
  0x7fffffffe098 : 0x0000000000000005  (5)
  0x7fffffffe0a0 : 0x00007fffffffe0b0          <-- RBP
  0x7fffffffe0a8 : 0x00000000004010a0
  ...
------------------------------------------------------------
[ Disassembly ]
=> 0x401028 <main+8>:  pop    rdi
   0x40102a <main+10>: pop    rax
   0x40102b <main+11>: add    rax,rdi
   0x40102e <main+14>: push   rax
   ...
------------------------------------------------------------
```

## 将来のアイディア

- **TUIモード対応** — `gdb.register_window_type()` (GDB 9.2+) を使い、カスタムTUIウィンドウとしてコンテキストを表示する。`tui new-layout` でソース/ディスアセンブリと並列表示が可能。ただしANSIカラーが使えない制約あり、Rosettaリモートスタブとの相性にも注意
- **TUI状態履歴** — TUI版ではスクロールバッファが使えなくなるため、状態のリングバッファを持たせて `prev` / `next` コマンドで過去のコンテキストに遡れるようにする。gef/pwndbg/pedaにもない機能であり、差別化要素になり得る
- **FLAGSレジスタの展開表示** — CF, ZF, SF, OF 等のフラグを個別に表示
- **設定のカスタマイズ機構** — 表示するレジスタ、スタック行数のデフォルト値、色の変更など
- **ウォッチポイント連携**
- **汎用化・別プロジェクト化** — 十分に育ったら「軽量なgef代替」として独立リポジトリに分離し公開を検討
