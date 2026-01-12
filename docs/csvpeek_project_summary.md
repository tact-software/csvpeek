# csvpeek / cpk — Project Summary

## Overview

**csvpeek** is a fast, memory-efficient CSV inspection and summarization CLI tool.  
The executable command is **`cpk`**, designed for frequent daily use with minimal typing.

The tool allows users to:
- Select columns explicitly
- Filter rows using simple expressions
- Compute summaries and statistics
- Handle very large CSV files safely via streaming

All operations are performed **without SQL, without pipes, and without loading entire files into memory**.

This document serves as a **high-level project summary** and is intended to be used together with the detailed specification.

---

## Name & Command

- **Project name**: `csvpeek`
- **CLI command**: `cpk`
- **Tagline**: *Fast CSV insights from the command line*

Example:
```bash
cpk data.csv --cols age,income --where 'age >= 30'
```

---

## Core Philosophy

### Design Goals
- **Speed first**: optimized for large files
- **Low memory usage**: streaming, no full materialization
- **CLI-native UX**: everything via arguments
- **Safe by default**: guardrails for large data and malformed rows
- **Readable results**: human-friendly output with machine-friendly options

### Non-Goals
- Full SQL compatibility
- Multi-file joins
- GUI or interactive TUI

---

## Key Features

### 1. Column-Focused Summaries
Users explicitly choose columns to inspect.

```bash
cpk data.csv --cols age,income
```

Avoids accidental heavy scans and keeps intent clear.

---

### 2. Argument-Based Filtering

Simple expressions instead of SQL:

```bash
cpk data.csv --where 'country == "JP" && age >= 30'
```

Supports:
- Comparisons (`== != < > <= >=`)
- Logical operators (`&& || !`)
- Null checks
- String helpers (`contains`, `startswith`, `in`, etc.)

---

### 3. Streaming & Scalability

- Processes CSV row by row
- Designed for multi-GB files
- Optional sampling and row limits
- Parallel execution where safe

---

### 4. Practical Statistics

Focused on statistics that matter during exploration:

- count / null_rate
- min / max / mean
- approximate percentiles
- string length stats
- optional top-k values

Approximate results are clearly marked.

---

### 5. Schema Inspection

Quickly understand unknown CSVs:

```bash
cpk schema data.csv
```

Outputs:
- Column names
- Inferred types
- Null ratios
- Parse success rates

---

## Typical Use Cases

- First look at an unfamiliar CSV file
- Sanity-checking exports and logs
- Lightweight data profiling before loading into DBs
- CI / automation checks on CSV outputs
- Ad-hoc analysis without Python or SQL

---

## Target Users

- Data engineers
- Backend engineers
- SRE / DevOps
- Analysts who prefer CLI tools
- Anyone dealing with large CSV files regularly

---

## Technical Direction

- Language: **Rust**
- Parsing: streaming CSV reader
- Concurrency: scoped parallelism
- Distribution: single static binary
- Platforms: Linux / macOS / Windows

---

## Relationship to Specification

This document:
- Explains **what csvpeek / cpk is**
- Clarifies **design intent and scope**

The specification document:
- Defines **exact CLI options**
- Describes **syntax and behavior**
- Acts as the implementation contract

Both documents are expected to evolve together.

---

## License

Planned:
- MIT or Apache-2.0

---

## One-Line Pitch

> **cpk** lets you peek into massive CSV files and get useful summaries in seconds — safely, simply, and from the command line.
