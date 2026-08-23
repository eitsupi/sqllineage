mod common;

use common::{analyze_one, concrete_sources, find_mapping};
use sqllineage::{AnalyzeOptions, Dialect, TransformKind, analyze};

fn analyze_with_dialect(sql: &str, dialect: Dialect) -> sqllineage::AnalyzeResult {
    analyze(
        sql,
        AnalyzeOptions {
            dialect,
            ..AnalyzeOptions::default()
        },
    )
    .expect("SQL should parse")
    .into_iter()
    .next()
    .unwrap_or_default()
}

#[test]
fn extract_year() {
    let result = analyze_one("SELECT EXTRACT(YEAR FROM hire_date) AS yr FROM t");
    let m = find_mapping(&result.columns.mappings, "yr");
    assert_eq!(concrete_sources(m), vec![("t".into(), "hire_date".into())]);
    assert_eq!(m.transform, TransformKind::Expression);
}

#[test]
fn ceil_expr() {
    let result = analyze_one("SELECT CEIL(price) AS p FROM t");
    let m = find_mapping(&result.columns.mappings, "p");
    assert_eq!(concrete_sources(m), vec![("t".into(), "price".into())]);
}

#[test]
fn floor_expr() {
    let result = analyze_one("SELECT FLOOR(price) AS p FROM t");
    let m = find_mapping(&result.columns.mappings, "p");
    assert_eq!(concrete_sources(m), vec![("t".into(), "price".into())]);
}

#[test]
fn substring_expr() {
    let result = analyze_one("SELECT SUBSTRING(name FROM 1 FOR 3) AS sub FROM t");
    let m = find_mapping(&result.columns.mappings, "sub");
    assert_eq!(concrete_sources(m), vec![("t".into(), "name".into())]);
}

#[test]
fn trim_expr() {
    let result = analyze_one("SELECT TRIM(name) AS n FROM t");
    let m = find_mapping(&result.columns.mappings, "n");
    assert_eq!(concrete_sources(m), vec![("t".into(), "name".into())]);
}

#[test]
fn position_expr() {
    let result = analyze_one("SELECT POSITION('x' IN name) AS pos FROM t");
    let m = find_mapping(&result.columns.mappings, "pos");
    assert_eq!(concrete_sources(m), vec![("t".into(), "name".into())]);
}

#[test]
fn overlay_expr() {
    let result = analyze_one("SELECT OVERLAY(name PLACING 'X' FROM 1 FOR 1) AS o FROM t");
    let m = find_mapping(&result.columns.mappings, "o");
    assert_eq!(concrete_sources(m), vec![("t".into(), "name".into())]);
}

#[test]
fn at_time_zone() {
    let result = analyze_one("SELECT ts AT TIME ZONE 'UTC' AS utc FROM t");
    let m = find_mapping(&result.columns.mappings, "utc");
    assert_eq!(concrete_sources(m), vec![("t".into(), "ts".into())]);
}

#[test]
fn collate_expr() {
    let result = analyze_one("SELECT name COLLATE \"en_US\" AS n FROM t");
    let m = find_mapping(&result.columns.mappings, "n");
    assert_eq!(concrete_sources(m), vec![("t".into(), "name".into())]);
}

#[test]
fn is_true_expr() {
    let result = analyze_one("SELECT active IS TRUE AS flag FROM t");
    let m = find_mapping(&result.columns.mappings, "flag");
    assert_eq!(concrete_sources(m), vec![("t".into(), "active".into())]);
}

#[test]
fn is_distinct_from() {
    let result = analyze_one("SELECT a IS DISTINCT FROM b AS diff FROM t");
    let m = find_mapping(&result.columns.mappings, "diff");
    assert_eq!(
        concrete_sources(m),
        vec![("t".into(), "a".into()), ("t".into(), "b".into())]
    );
}

#[test]
fn like_expr() {
    let result = analyze_one("SELECT name LIKE '%test%' AS matched FROM t");
    let m = find_mapping(&result.columns.mappings, "matched");
    assert_eq!(concrete_sources(m), vec![("t".into(), "name".into())]);
}

#[test]
fn array_expr() {
    let result = analyze_one("SELECT ARRAY[a, b] AS arr FROM t");
    let m = find_mapping(&result.columns.mappings, "arr");
    assert_eq!(
        concrete_sources(m),
        vec![("t".into(), "a".into()), ("t".into(), "b".into())]
    );
}

#[test]
fn json_access() {
    let result = analyze_one("SELECT data->>'key' AS val FROM t");
    let m = find_mapping(&result.columns.mappings, "val");
    assert_eq!(concrete_sources(m), vec![("t".into(), "data".into())]);
}

#[test]
fn qualified_compound_field_access_uses_binding_column() {
    let result = analyze_one("SELECT base.items_array[1] AS item FROM actual_table AS base");
    let m = find_mapping(&result.columns.mappings, "item");
    assert_eq!(
        concrete_sources(m),
        vec![("actual_table".into(), "items_array".into())]
    );
}

#[test]
fn compound_field_access_retains_column_dependent_index() {
    let result = analyze_one("SELECT base.items_array[idx] AS item FROM actual_table AS base");
    let m = find_mapping(&result.columns.mappings, "item");
    assert_eq!(
        concrete_sources(m),
        vec![
            ("actual_table".into(), "idx".into()),
            ("actual_table".into(), "items_array".into()),
        ]
    );
}

#[test]
fn nested_qualified_compound_field_access_keeps_top_level_column() {
    let result = analyze_one("SELECT base.payload.items[1] AS item FROM actual_table AS base");
    let m = find_mapping(&result.columns.mappings, "item");
    assert_eq!(
        concrete_sources(m),
        vec![("actual_table".into(), "payload".into())]
    );
}

#[test]
fn cte_compound_field_access_uses_cte_binding_column() {
    let result = analyze_one(
        "WITH base AS (SELECT items_array FROM actual_table) SELECT base.items_array[1] AS item FROM base",
    );
    let m = find_mapping(&result.columns.mappings, "item");
    assert_eq!(
        concrete_sources(m),
        vec![("actual_table".into(), "items_array".into())]
    );
}

#[test]
fn unqualified_compound_field_access_uses_top_level_column() {
    let result = analyze_one("SELECT payload.items[1] AS item FROM t");
    let m = find_mapping(&result.columns.mappings, "item");
    assert_eq!(concrete_sources(m), vec![("t".into(), "payload".into())]);
}

#[test]
fn bigquery_offset_compound_field_access_uses_binding_column() {
    let result = analyze_with_dialect(
        "SELECT base.items_array[OFFSET(0)] AS item FROM actual_table AS base",
        Dialect::BigQuery,
    );
    let m = find_mapping(&result.columns.mappings, "item");
    assert_eq!(
        concrete_sources(m),
        vec![("actual_table".into(), "items_array".into())]
    );
}

#[test]
fn bigquery_date_trunc_week_modifier_is_syntax_only() {
    let result = analyze_with_dialect(
        "SELECT DATE_TRUNC(event_date, WEEK(MONDAY)) AS monday_start, DATE_TRUNC(event_date, WEEK(SUNDAY)) AS sunday_start FROM events",
        Dialect::BigQuery,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "monday_start")),
        vec![("events".into(), "event_date".into())]
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "sunday_start")),
        vec![("events".into(), "event_date".into())]
    );
}

#[test]
fn bigquery_date_diff_isoweek_keeps_only_date_values() {
    let result = analyze_with_dialect(
        "SELECT DATE_DIFF(event_date, other_date, ISOWEEK) AS days FROM events",
        Dialect::BigQuery,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "days")),
        vec![
            ("events".into(), "event_date".into()),
            ("events".into(), "other_date".into()),
        ]
    );
}

#[test]
fn generic_date_trunc_keeps_date_part_identifiers() {
    let result =
        analyze_one("SELECT DATE_TRUNC(event_date, WEEK(MONDAY)) AS week_start FROM events");
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "week_start")),
        vec![
            ("events".into(), "MONDAY".into()),
            ("events".into(), "event_date".into()),
        ]
    );
}

#[test]
fn bigquery_date_trunc_isoyear_and_timezone_data_are_classified() {
    let result = analyze_with_dialect(
        "SELECT DATE_TRUNC(event_date, ISOYEAR) AS year_start, TIMESTAMP_TRUNC(event_ts, DAY, tz_name) AS ts_day FROM events",
        Dialect::BigQuery,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "year_start")),
        vec![("events".into(), "event_date".into())]
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "ts_day")),
        vec![
            ("events".into(), "event_ts".into()),
            ("events".into(), "tz_name".into()),
        ]
    );
}

#[test]
fn bigquery_date_trunc_three_argument_form_is_not_a_timezone_signature() {
    let result = analyze_with_dialect(
        "SELECT DATE_TRUNC(event_date, DAY, tz_name) AS day_start FROM events",
        Dialect::BigQuery,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "day_start")),
        vec![
            ("events".into(), "DAY".into()),
            ("events".into(), "event_date".into()),
            ("events".into(), "tz_name".into()),
        ]
    );
}

#[test]
fn bigquery_date_part_position_can_still_be_a_data_expression() {
    let result = analyze_with_dialect(
        "SELECT DATE_TRUNC(WEEK, WEEK(MONDAY)) AS week_start FROM events",
        Dialect::BigQuery,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "week_start")),
        vec![("events".into(), "WEEK".into())]
    );
}

#[test]
fn bigquery_last_day_has_optional_static_part() {
    let result = analyze_with_dialect(
        "SELECT LAST_DAY(event_date, MONTH) AS month_end, LAST_DAY(event_date) AS day_end FROM events",
        Dialect::BigQuery,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "month_end")),
        vec![("events".into(), "event_date".into())]
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "day_end")),
        vec![("events".into(), "event_date".into())]
    );
}

#[test]
fn unknown_udf_keeps_all_arguments() {
    let result = analyze_with_dialect(
        "SELECT my_udf(event_date, ISOYEAR) AS udf_value FROM events",
        Dialect::BigQuery,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "udf_value")),
        vec![
            ("events".into(), "ISOYEAR".into()),
            ("events".into(), "event_date".into()),
        ]
    );
}

#[test]
fn snowflake_date_part_signatures_skip_only_static_parts() {
    let result = analyze_with_dialect(
        "SELECT DATEADD(DAY, amount, event_date) AS added, DATE_PART(YEAR, event_date) AS year_value, TRUNC(event_date, dynamic_part) AS truncated FROM events",
        Dialect::Snowflake,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "added")),
        vec![
            ("events".into(), "amount".into()),
            ("events".into(), "event_date".into()),
        ]
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "year_value")),
        vec![("events".into(), "event_date".into())]
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "truncated")),
        vec![
            ("events".into(), "dynamic_part".into()),
            ("events".into(), "event_date".into()),
        ]
    );
}

#[test]
fn mysql_and_databricks_date_part_signatures_are_dialect_scoped() {
    let mysql = analyze_with_dialect(
        "SELECT TIMESTAMPDIFF(DAY, start_date, end_date) AS elapsed FROM events",
        Dialect::MySql,
    );
    assert_eq!(
        concrete_sources(find_mapping(&mysql.columns.mappings, "elapsed")),
        vec![
            ("events".into(), "end_date".into()),
            ("events".into(), "start_date".into()),
        ]
    );

    let databricks = analyze_with_dialect(
        "SELECT DATEDIFF(DAY, start_date, end_date) AS elapsed FROM events",
        Dialect::Databricks,
    );
    assert_eq!(
        concrete_sources(find_mapping(&databricks.columns.mappings, "elapsed")),
        vec![
            ("events".into(), "end_date".into()),
            ("events".into(), "start_date".into()),
        ]
    );
}

#[test]
fn mysql_date_part_aliases_are_limited_to_legal_timestamp_units() {
    let result = analyze_with_dialect(
        "SELECT TIMESTAMPDIFF(SQL_TSI_DAY, start_date, end_date) AS aliased, TIMESTAMPDIFF(DAY_SECOND, start_date, end_date) AS composite FROM events",
        Dialect::MySql,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "aliased")),
        vec![
            ("events".into(), "end_date".into()),
            ("events".into(), "start_date".into()),
        ]
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "composite")),
        vec![
            ("events".into(), "DAY_SECOND".into()),
            ("events".into(), "end_date".into()),
            ("events".into(), "start_date".into()),
        ]
    );
}

#[test]
fn databricks_add_and_diff_have_distinct_date_part_grammars() {
    let result = analyze_with_dialect(
        "SELECT DATEADD(DAYOFYEAR, amount, event_ts) AS added, DATEDIFF(DAYOFYEAR, start_ts, end_ts) AS elapsed FROM events",
        Dialect::Databricks,
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "added")),
        vec![
            ("events".into(), "amount".into()),
            ("events".into(), "event_ts".into()),
        ]
    );
    assert_eq!(
        concrete_sources(find_mapping(&result.columns.mappings, "elapsed")),
        vec![
            ("events".into(), "DAYOFYEAR".into()),
            ("events".into(), "end_ts".into()),
            ("events".into(), "start_ts".into()),
        ]
    );
}
