use std::sync::{Arc, Mutex};

use calamine::{open_workbook_auto, Data, DataType, Range, Reader};
use chrono::{Duration, NaiveDate, NaiveDateTime};
use deunicode::deunicode;
use heck::ToSnakeCase;
use mysql::{prelude::Queryable, Pool, Value};

use crate::{
    errors::{AppError, AppResult},
    ActionData,
};

pub async fn add_exel_table(action_data: ActionData, pool: Arc<Mutex<Pool>>) -> AppResult<()> {
    let exel_data = get_exel_file_data(&action_data).await?;
    let new_table = create_table(
        &exel_data.headers,
        &exel_data
            .rows
            .get(0)
            .ok_or(AppError::Db("Невдалося прочитати заголовки таблиці"))?,
        pool.clone(),
        &exel_data.table_name,
    )
    .await?;

    write_rows(&exel_data.headers, &new_table, exel_data.rows, pool).await?;
    Ok(())
}

pub async fn append_table(action_data: ActionData, pool: Arc<Mutex<Pool>>) -> AppResult<()> {
    let exel_data = get_exel_file_data(&action_data).await?;
    write_rows(
        &exel_data.headers,
        &action_data.table_name,
        exel_data.rows,
        pool,
    )
    .await?;
    Ok(())
}

pub async fn rewrite_table(pool: Arc<Mutex<Pool>>, action_data: ActionData) -> AppResult<()> {
    let mut conn = pool
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| AppError::DbConnErr)?;
    let query = format!("TRUNCATE TABLE `{}`", action_data.table_name);
    conn.query_drop(query)
        .map_err(|_| AppError::Db("Невдалося видалити таблицю"))?;
    let exel_data = get_exel_file_data(&action_data).await?;
    write_rows(
        &exel_data.headers,
        &action_data.table_name,
        exel_data.rows,
        pool,
    )
    .await?;
    Ok(())
}

pub struct ExelData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Data>>,
    pub table_name: String,
}

pub async fn get_exel_file_data(action_data: &ActionData) -> AppResult<ExelData> {
    let mut workbook = open_workbook_auto(&action_data.file_path)
        .map_err(|_| AppError::Exel("Невдалося выдкрити файл"))?;
    let _ = workbook
        .sheet_names()
        .iter()
        .find(|&x| *x == action_data.sheet)
        .ok_or(AppError::Exel("Невдалося знайти обраний аркуш"))?;
    let range = workbook
        .worksheet_range(&action_data.sheet)
        .map_err(|_| AppError::Exel("Невірно обраний діапазон"))?;
    let (headers, rows) = get_header_rows(range, &action_data.range).await?;
    let table_name = deunicode(&action_data.table_name).to_snake_case();
    let headers: Vec<String> = headers
        .iter()
        .map(|val| deunicode(val).to_snake_case())
        .collect();
    Ok(ExelData {
        headers,
        rows,
        table_name,
    })
}

pub async fn write_rows(
    headers: &[String],
    table_name: &str,
    rows: Vec<Vec<Data>>,
    pool: Arc<Mutex<Pool>>,
) -> AppResult<()> {
    let placeholders = headers.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let insert_query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        headers.join(", "),
        placeholders
    );
    let mut conn = pool
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| AppError::DbConnErr)?;
    let stmt = conn.prep(&insert_query).unwrap();
    for row in &rows {
        let values: Vec<Value> = row
            .iter()
            .map(|cell| match cell {
                Data::Int(i) => Value::from(*i),
                Data::Float(f) => Value::from(*f),
                Data::String(s) => Value::from(s.as_str()),
                Data::Bool(b) => Value::from(*b),
                Data::DateTime(dt) => Value::from(excel_serial_to_mysql_datetime(dt.as_f64())),
                _ => Value::NULL,
            })
            .collect();
        conn.exec_drop(&stmt, &values)
            .map_err(|_| AppError::Db("Невдалося додати данні в таблицю"))?;
    }
    Ok(())
}

pub async fn delete_table(table_name: &str, pool: Arc<Mutex<Pool>>) -> AppResult<()> {
    let mut conn = pool
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| AppError::DbConnErr)?;
    let query = format!("DROP TABLE IF EXISTS `{}`", table_name);
    conn.query_drop(query)
        .map_err(|_| AppError::Db("Невдалося видалити таблицю"))?;
    Ok(())
}

async fn create_table(
    headers: &[String],
    first_row: &[Data],
    pool: Arc<Mutex<Pool>>,
    table_name: &str,
) -> AppResult<String> {
    let columns: Vec<String> = headers
        .iter()
        .zip(first_row.iter())
        .map(|(name, cell)| {
            let sql_type = match cell {
                Data::Int(_) => "INT",
                Data::Bool(_) => "BOOLEAN",
                Data::String(_) => "VARCHAR(255)",
                Data::Float(_) => "DOUBLE",
                Data::DateTime(_) => "DATETIME",
                _ => "TEXT",
            };
            let clean_name = name;
            format!("`{}` {}", clean_name, sql_type)
        })
        .collect();
    let create_query = format!(
        "CREATE TABLE IF NOT EXISTS {} (id INT AUTO_INCREMENT PRIMARY KEY, {})",
        table_name,
        columns.join(", ")
    );
    let mut conn = pool
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| AppError::DbConnErr)?;
    conn.query_drop(&create_query)
        .map_err(|_| AppError::Exel("Не вдалося створити таблицю"))?;
    Ok(table_name.to_string())
}

async fn get_header_rows(
    range: Range<Data>,
    table_range: &str,
) -> AppResult<(Vec<String>, Vec<Vec<Data>>)> {
    let (start, end) = parse_range(table_range)?;
    let range = range.range(start, end);
    let mut headers: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<Data>> = Vec::new();
    let mut is_first_row = true;
    for row in range.rows() {
        if is_first_row {
            headers = row
                .iter()
                .map(|r| r.get_string().unwrap_or("unknown").to_string())
                .collect();
            is_first_row = false;
            continue;
        }
        rows.push(row.to_vec());
    }
    Ok((headers, rows))
}

fn excel_serial_to_mysql_datetime(serial: f64) -> Option<String> {
    if !serial.is_finite() {
        return None;
    }

    // Excel epoch starts at 1899-12-30 (due to leap-year bug)
    let base_date = NaiveDate::from_ymd_opt(1899, 12, 30)?;

    let days = serial.trunc() as i64;
    let seconds = ((serial.fract()) * 86400.0).round() as i64;

    let dt: NaiveDateTime = base_date
        .and_hms_opt(0, 0, 0)?
        .checked_add_signed(Duration::days(days))?
        .checked_add_signed(Duration::seconds(seconds))?;

    Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn col_to_index(col: &str) -> AppResult<u32> {
    let mut index = 0;
    for (i, c) in col.chars().rev().enumerate() {
        let val = (c as u8 - b'A' + 1) as u32;
        index += val * 26_u32.pow(i as u32);
    }
    Ok(index
        .checked_sub(1)
        .ok_or(AppError::Exel("Невірний діапазон"))?) // zero-based
}

fn parse_cell(cell: &str) -> AppResult<(u32, u32)> {
    let (col_part, row_part) = cell.chars().partition::<String, _>(|c| c.is_alphabetic());
    let col_index = col_to_index(&col_part);
    let row_index = row_part
        .parse::<u32>()
        .map_err(|_| AppError::Exel("Невірний діапазон"))?
        - 1; // zero-based
    Ok((row_index, col_index?))
}

fn parse_range(range: &str) -> AppResult<((u32, u32), (u32, u32))> {
    let parts: Vec<&str> = range.split(':').collect();
    let start = parse_cell(parts[0]);
    let end = parse_cell(parts[1]);
    Ok((start?, end?))
}
