use std::process::Command;

fn csvp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_csvp"))
}

fn fixtures_path(name: &str) -> String {
    format!("tests/fixtures/{name}")
}

mod summary_command {
    use super::*;

    #[test]
    fn test_basic_summary() {
        let output = csvp()
            .arg(fixtures_path("basic.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("file: tests/fixtures/basic.csv"));
        assert!(stdout.contains("rows: 5"));
        assert!(stdout.contains("id"));
        assert!(stdout.contains("name"));
        assert!(stdout.contains("age"));
        assert!(stdout.contains("salary"));
        assert!(stdout.contains("active"));
    }

    #[test]
    fn test_summary_with_cols() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--cols")
            .arg("age,salary")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("age"));
        assert!(stdout.contains("salary"));
        // Should not contain other columns
        assert!(!stdout.contains("| name"));
    }

    #[test]
    fn test_summary_with_cols_by_index() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--cols")
            .arg("0,2")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("id"));
        assert!(stdout.contains("age"));
    }

    #[test]
    fn test_summary_with_filter() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("age > 30")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("matched: 2"));
    }

    #[test]
    fn test_summary_json_format() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("-f")
            .arg("json")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // JSON output should be parseable
        assert!(stdout.contains("["));
        assert!(stdout.contains("\"name\""));
        assert!(stdout.contains("\"data_type\""));
    }

    #[test]
    fn test_summary_csv_format() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("-f")
            .arg("csv")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("column,type,count"));
    }

    #[test]
    fn test_summary_ndjson_format() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("-f")
            .arg("ndjson")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Each line should be a JSON object
        for line in stdout.lines() {
            assert!(line.starts_with("{") || line.is_empty());
        }
    }

    #[test]
    fn test_summary_with_nulls() {
        let output = csvp()
            .arg(fixtures_path("with_nulls.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should show null rates
        assert!(stdout.contains("%"));
    }
}

mod schema_command {
    use super::*;

    #[test]
    fn test_schema_basic() {
        let output = csvp()
            .arg("schema")
            .arg(fixtures_path("basic.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("columns: 5"));
        assert!(stdout.contains("integer"));
        assert!(stdout.contains("string"));
        assert!(stdout.contains("boolean"));
    }

    #[test]
    fn test_schema_json_format() {
        let output = csvp()
            .arg("schema")
            .arg(fixtures_path("basic.csv"))
            .arg("-f")
            .arg("json")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("["));
        assert!(stdout.contains("\"inferred_type\""));
    }

    #[test]
    fn test_schema_with_sample_values() {
        let output = csvp()
            .arg("schema")
            .arg(fixtures_path("basic.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should show sample values like Alice, Bob
        assert!(stdout.contains("samples"));
    }
}

mod delimiter_options {
    use super::*;

    #[test]
    fn test_tab_delimiter() {
        let output = csvp()
            .arg("-d")
            .arg("tab")
            .arg(fixtures_path("tab_separated.tsv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("product"));
        assert!(stdout.contains("quantity"));
    }

    #[test]
    fn test_tab_delimiter_backslash() {
        let output = csvp()
            .arg("-d")
            .arg("\\t")
            .arg(fixtures_path("tab_separated.tsv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
    }
}

mod no_header_option {
    use super::*;

    #[test]
    fn test_no_header() {
        let output = csvp()
            .arg("--no-header")
            .arg(fixtures_path("no_header.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should generate col0, col1, col2, col3
        assert!(stdout.contains("col0"));
        assert!(stdout.contains("col1"));
        assert!(stdout.contains("col2"));
        assert!(stdout.contains("col3"));
    }
}

mod filter_expressions {
    use super::*;

    #[test]
    fn test_filter_equals() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("name == \"Alice\"")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("matched: 1"));
    }

    #[test]
    fn test_filter_greater_than() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("age >= 30")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("matched: 3"));
    }

    #[test]
    fn test_filter_contains() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("contains(name, \"a\")")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Charlie, Diana have 'a'
        assert!(stdout.contains("matched:"));
    }

    #[test]
    fn test_filter_in() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("in(name, [\"Alice\", \"Bob\"])")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("matched: 2"));
    }

    #[test]
    fn test_filter_is_null() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("with_nulls.csv"))
            .arg("--where")
            .arg("is_null(age)")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Bob and Diana have null ages
        assert!(stdout.contains("matched:"));
    }

    #[test]
    fn test_filter_logical_and() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("age > 25 && active == true")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
    }

    #[test]
    fn test_filter_matches_regex() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("matches(name, \"^[AB]\")")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Alice and Bob start with A or B
        assert!(stdout.contains("matched: 2"));
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_file_not_found() {
        let output = csvp()
            .arg("nonexistent.csv")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("not found") || stderr.contains("No such file"));
    }

    #[test]
    fn test_invalid_column() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--cols")
            .arg("nonexistent")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("not found") || stderr.contains("Column"));
    }

    #[test]
    fn test_column_index_out_of_range() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--cols")
            .arg("999")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("out of range") || stderr.contains("index"));
    }

    #[test]
    fn test_invalid_filter() {
        let output = csvp()
            .arg("summary")
            .arg(fixtures_path("basic.csv"))
            .arg("--where")
            .arg("invalid_column > 5")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }
}

mod quiet_option {
    use super::*;

    #[test]
    fn test_quiet_flag() {
        let output = csvp()
            .arg("-q")
            .arg(fixtures_path("basic.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        // Output should still contain results
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("file:"));
    }
}

mod output_option {
    use super::*;
    use std::fs;

    #[test]
    fn test_output_to_file() {
        let output_file = "/tmp/csvpeek_test_output.txt";

        let output = csvp()
            .arg("-o")
            .arg(output_file)
            .arg(fixtures_path("basic.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        // Check file was created and has content
        let content = fs::read_to_string(output_file).expect("Failed to read output file");
        assert!(content.contains("file:"));
        assert!(content.contains("rows:"));

        // Cleanup
        fs::remove_file(output_file).ok();
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_csv() {
        let output = csvp()
            .arg(fixtures_path("empty.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("rows: 0"));
    }

    #[test]
    fn test_single_row() {
        let output = csvp()
            .arg(fixtures_path("single_row.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("rows: 1"));
    }

    #[test]
    fn test_special_chars_in_csv() {
        let output = csvp()
            .arg(fixtures_path("special_chars.csv"))
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("rows: 4"));
    }
}
