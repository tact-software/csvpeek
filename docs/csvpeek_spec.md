# csvpeek (csvp) — 高速CSVサマリーCLI 仕様書

## 1. 概要

`csvp` は **巨大CSVをストリーム処理で高速に集計** できるCLIツール。
SQLやパイプを使わず、**CLI引数のみで列指定・フィルター・サマリー**を完結させることを目的とする。

- **プロジェクト名**: csvpeek
- **CLIコマンド**: csvp

---

## 2. 目的と非目的

### 目的
- 巨大CSVでも低メモリで動作
- 列指定 + フィルター + サマリーを1コマンドで実行
- 人が書きやすく、事故りにくいCLI UX

### 非目的（MVPでは対象外）
- JOIN / 複数CSV結合
- SQL完全互換
- GUI

---

## 3. コマンド構成

```bash
csvp <COMMAND> <FILE> [options]
```

### サブコマンド
- `summary`（デフォルト）
- `schema`（列名・型推論・null率確認）

例:
```bash
csvp summary data.csv --cols age,income
csvp schema data.csv
```

---

## 4. 入力仕様

### FILE
- 通常ファイル or `-`（stdin）
- `.gz` は自動解凍

### CSVオプション
- `--delimiter ','`
- `--header true|false`（default: true）
- `--quote '"'`
- `--trim`

---

## 5. 出力仕様

### 出力形式
- `table`（default）
- `json`
- `ndjson`
- `csv`

### table例
```
file: data.csv
rows: 1200345 (matched: 234221)
filter: age>=30 && country=="JP"
----------------------------------------------------
column   type   count   null%   min   max   mean
age      i64    ...     ...     ...   ...   ...
income   f64    ...     ...     ...   ...   ...
```

---

## 6. 列指定

- `--cols age,income`
- `--cols @all`
- `--exclude password,token`

### 列参照
- ヘッダあり: `age`
- ヘッダなし: `#1`（1始まり）

---

## 7. フィルター（--where）

### 例
```bash
--where 'age>=30 && country=="JP"'
--where 'contains(email,"@example.com")'
```

### 演算子
- 比較: `== != < <= > >=`
- 論理: `&& || !`
- null: `null`
- 括弧: `( )`

### 関数（MVP）
- `contains(col,str)`
- `startswith(col,str)`
- `endswith(col,str)`
- `regex(col,pattern)`
- `in(col,[...])`
- `is_null(col)` / `not_null(col)`

※ 型不一致はエラーにせず、その行を除外 + 警告カウント

---

## 8. サマリー統計

### 指定
```bash
--stats count,null_rate,min,max,mean
```

### 数値列
- count
- null_count / null_rate
- min / max
- mean
- median（近似）
- p50 / p90 / p95 / p99（近似）

### 文字列列
- count
- null_rate
- min_len / max_len / avg_len
- distinct（近似）
- topk:N

### 近似について
- デフォルトは近似
- `--exact` 指定で厳密（遅い）
- 出力時は `≈` を付与

---

## 9. グループ集計

```bash
--group-by country
--limit-groups 100
```

---

## 10. パフォーマンス・安全装置

- `--threads N`
- `--sample N`
- `--max-rows N`
- `--max-field-bytes N`
- `--bad-rows skip|error|count`
- `--progress`

---

## 11. 型推論

### schema出力
- 列名
- 推定型: i64 / f64 / bool / date / datetime / string
- パース成功率
- null率

### オプション
- `--infer-rows N`
- `--type age:i64,income:f64`

---

## 12. エラーUX

- 列名ミス → 候補提示
- where構文エラー → 位置付き表示
- 型不一致 → 件数を警告表示

例:
```
error: unknown column "contry" (did you mean "country"?)
```

---

## 13. MVP最小構成

- summary
- --cols
- --where（比較 + 論理 + contains/in/null）
- stats: count,null_rate,min,max,mean
- 出力: table / json

---

## 14. README用ユースケース

```bash
csvp data.csv --cols age,income
csvp data.csv --cols income --where 'age>=30 && country=="JP"'
csvp data.csv --group-by country --cols income --stats mean,count
csvp schema data.csv
```

---

## 15. ライセンス想定
- MIT or Apache-2.0
