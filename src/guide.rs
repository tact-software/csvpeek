/// Topic-based detailed help system
pub fn print_guide(topic: Option<&str>) {
    match topic {
        Some("filters") | Some("filter") => print_filters_guide(),
        Some("stats") | Some("statistics") => print_stats_guide(),
        Some("columns") | Some("cols") => print_columns_guide(),
        Some("formats") | Some("format") => print_formats_guide(),
        Some("encoding") | Some("encodings") => print_encoding_guide(),
        Some(unknown) => {
            eprintln!("Unknown topic: {unknown}");
            eprintln!();
            print_topics_list();
        }
        None => print_topics_list(),
    }
}

fn print_topics_list() {
    println!(
        r#"csvp guide - Detailed help for specific topics

USAGE:
    csvp guide <TOPIC>

AVAILABLE TOPICS:
    filters     Filter expression syntax and examples
    stats       Available statistics for each column type
    columns     Column specification syntax (names, indices, ranges)
    formats     Output format details (table, json, ndjson, csv)
    encoding    Supported character encodings

EXAMPLES:
    csvp guide filters
    csvp guide stats
"#
    );
}

fn print_filters_guide() {
    println!(
        r#"FILTER EXPRESSIONS

Filter expressions allow you to analyze only rows matching specific conditions.
Use with: csvp data.csv -w "<expression>"

COMPARISON OPERATORS:
    ==          Equal               name == "Alice"
    !=          Not equal           status != "inactive"
    >           Greater than        age > 30
    >=          Greater or equal    price >= 100
    <           Less than           score < 50
    <=          Less or equal       quantity <= 10

LOGICAL OPERATORS:
    &&          AND                 age > 20 && age < 30
    ||          OR                  status == "A" || status == "B"
    !           NOT                 !(status == "deleted")

FUNCTIONS:
    contains(column, "text")        String contains substring
        Example: contains(name, "test")

    matches(column, "regex")        Regular expression match
        Example: matches(email, "@example\\.com$")
        Example: matches(id, "^[A-Z]\\d{{4}}$")

    in(column, ["a", "b", "c"])     Value in list
        Example: in(status, ["active", "pending", "review"])

    is_null(column)                 Value is null/empty
        Example: is_null(email)

    is_not_null(column)             Value is not null/empty
        Example: is_not_null(phone)

GROUPING:
    Use parentheses for complex expressions:
        (age > 25 && age < 35) || status == "VIP"
        !(status == "deleted" || status == "archived")

STRING VALUES:
    Wrap string values in double quotes:
        name == "Alice"
        contains(description, "urgent")

    Escape quotes inside strings:
        message == "Say \"hello\""

NUMERIC VALUES:
    Numbers do not need quotes:
        age > 30
        price >= 99.99

EXAMPLES:
    # Simple comparison
    csvp data.csv -w "age > 30"

    # String equality
    csvp data.csv -w "status == \"active\""

    # Combined conditions
    csvp data.csv -w "age >= 18 && age <= 65 && is_not_null(email)"

    # Using functions
    csvp data.csv -w "contains(name, \"Corp\") && in(region, [\"US\", \"EU\"])"

    # Complex grouping
    csvp data.csv -w "(type == \"A\" || type == \"B\") && price > 100"
"#
    );
}

fn print_stats_guide() {
    println!(
        r#"STATISTICS

The summary command computes various statistics for each column based on its data type.

COMMON STATISTICS (all columns):
    count       Number of non-null values
    null%       Percentage of null/empty values
    unique      Number of unique values

NUMERIC COLUMNS (Integer, Float):
    min         Minimum value
    max         Maximum value
    mean        Arithmetic mean (average)
    median      Middle value (50th percentile)
    std         Standard deviation
    p25         25th percentile (first quartile)
    p75         75th percentile (third quartile)

STRING COLUMNS:
    min_len     Minimum string length
    max_len     Maximum string length
    top         Most frequent values (up to 5)

BOOLEAN COLUMNS:
    Treated as string columns showing true/false distribution

DATA TYPE INFERENCE:
    Integer     All non-null values parse as integers
    Float       Values contain decimals or mix of int/float
    Boolean     All values are true/false (case-insensitive)
    String      Everything else

OUTPUT COLUMNS BY FORMAT:
    table       column, type, count, null%, unique, min, max, mean, median, std
    csv/json    All statistics including: p25, p75, sum, min_len, max_len

EXAMPLES:
    # View all statistics for all columns
    csvp data.csv

    # View statistics for specific columns
    csvp data.csv -c "price,quantity,total"

    # Export full statistics to JSON
    csvp data.csv -f json -o stats.json

    # Filter before computing statistics
    csvp data.csv -w "year == 2024" -c "revenue,profit"
"#
    );
}

fn print_columns_guide() {
    println!(
        r#"COLUMN SPECIFICATION

The -c/--cols option allows you to specify which columns to analyze.

BY NAME:
    -c "name"               Single column
    -c "name,age,city"      Multiple columns (comma-separated)

BY INDEX (0-based):
    -c "0"                  First column
    -c "0,1,2"              First three columns

BY RANGE:
    -c "0..5"               Columns 0,1,2,3,4 (exclusive end)
    -c "0..=5"              Columns 0,1,2,3,4,5 (inclusive end)
    -c "3..7"               Columns 3,4,5,6

MIXED:
    -c "name,0,3..5"        Combine names, indices, and ranges

NOTES:
    - Column indices start at 0
    - Range syntax follows Rust conventions:
        0..5  = 0,1,2,3,4 (end excluded)
        0..=5 = 0,1,2,3,4,5 (end included)
    - If a column name looks like a number, it's tried as an index first
    - Unknown column names show suggestions for similar names

EXAMPLES:
    # Analyze first 10 columns
    csvp data.csv -c "0..10"

    # Analyze specific named columns
    csvp data.csv -c "customer_id,order_date,total_amount"

    # Mix of names and ranges
    csvp data.csv -c "id,1..5,description"

    # All columns from index 5 onwards (use total column count - 1)
    csvp data.csv -c "5..=20"

ERROR HANDLING:
    - Invalid index: "Column index 99 is out of range (max: 10)"
    - Unknown name:  "Column 'nmae' not found. Did you mean: 'name'?"
"#
    );
}

fn print_formats_guide() {
    println!(
        r#"OUTPUT FORMATS

Use -f/--format to specify output format. Default is 'table'.

TABLE (default):
    Human-readable formatted table with colors (when terminal supports it).
    Best for interactive use.

    csvp data.csv -f table

    Example output:
    ┌──────────┬─────────┬───────┬───────┬────────┬─────┬─────┬───────┐
    │ column   │ type    │ count │ null% │ unique │ min │ max │ mean  │
    ├──────────┼─────────┼───────┼───────┼────────┼─────┼─────┼───────┤
    │ age      │ Integer │ 1000  │ 0.0%  │ 80     │ 18  │ 95  │ 42.5  │
    └──────────┴─────────┴───────┴───────┴────────┴─────┴─────┴───────┘

JSON:
    Pretty-printed JSON array. Good for programmatic processing.

    csvp data.csv -f json

    Example output:
    [
      {{
        "column": "age",
        "data_type": "Integer",
        "count": 1000,
        ...
      }}
    ]

NDJSON (Newline Delimited JSON):
    One JSON object per line. Best for streaming/large datasets.

    csvp data.csv -f ndjson

    Example output:
    {{"column":"age","data_type":"Integer","count":1000,...}}
    {{"column":"name","data_type":"String","count":1000,...}}

CSV:
    CSV format with all statistics. Good for spreadsheets/further analysis.

    csvp data.csv -f csv

    Includes additional columns not shown in table format:
    - p25, p75 (percentiles)
    - sum (total)
    - min_len, max_len (string lengths)

OUTPUT TO FILE:
    Use -o/--output to write to a file instead of stdout:

    csvp data.csv -f json -o stats.json
    csvp data.csv -f csv -o report.csv

COLOR CONTROL:
    Use --color to control colored output:
    --color auto      Auto-detect terminal (default)
    --color always    Always use colors
    --color never     Never use colors
"#
    );
}

fn print_encoding_guide() {
    println!(
        r#"CHARACTER ENCODING

By default, csvp auto-detects the file encoding. Use -e/--encoding to specify manually.

AUTO-DETECTION:
    1. Check for BOM (Byte Order Mark) - UTF-8, UTF-16 LE/BE
    2. Try UTF-8 decoding
    3. Use chardetng library for detection

SUPPORTED ENCODINGS:

    Unicode:
        utf-8, utf8             UTF-8 (most common)
        utf-16le                UTF-16 Little Endian
        utf-16be                UTF-16 Big Endian

    Japanese:
        shift_jis, sjis, cp932  Shift JIS (Windows Japanese)
        euc-jp                  EUC-JP (Unix Japanese)
        iso-2022-jp             ISO-2022-JP (Email Japanese)

    Chinese:
        gbk, gb2312, cp936      GBK (Simplified Chinese)
        gb18030                 GB18030 (Chinese national standard)
        big5, cp950             Big5 (Traditional Chinese)

    Korean:
        euc-kr, cp949           EUC-KR (Korean)

    Western European:
        latin1, iso-8859-1      Latin-1
        cp1252                  Windows-1252
        iso-8859-15             Latin-9 (with Euro sign)

    Eastern European:
        iso-8859-2              Latin-2

    Cyrillic:
        koi8-r                  KOI8-R (Russian)
        koi8-u                  KOI8-U (Ukrainian)

EXAMPLES:
    # Auto-detect encoding (default)
    csvp data.csv

    # Specify Shift_JIS for Japanese file
    csvp data.csv -e shift_jis

    # Specify GBK for Chinese file
    csvp data.csv -e gbk

    # UTF-16 with BOM
    csvp data.csv -e utf-16le

TIPS:
    - Most modern files are UTF-8
    - Japanese files from Windows are often Shift_JIS
    - If you see garbled text, try specifying the encoding explicitly
    - Export from Excel often uses cp1252 (Windows) or UTF-8 with BOM
"#
    );
}
