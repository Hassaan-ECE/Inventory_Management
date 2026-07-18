use std::{fs, path::Path};

use calamine::{open_workbook_auto, Data, DataType, Reader, SheetVisible};
use sha2::{Digest, Sha256};

use crate::model::CommandResult;

#[derive(Debug, Clone)]
pub(super) struct ParsedSource {
    pub filename: String,
    pub selected_sheet: String,
    pub fingerprint: String,
    pub headers: Vec<String>,
    pub unheaded_columns: Vec<usize>,
    pub rows: Vec<ParsedRow>,
}

#[derive(Debug, Clone)]
pub(super) struct ParsedRow {
    pub source_row: u64,
    pub values: Vec<String>,
}

pub(super) fn parse_source(path: &Path) -> CommandResult<ParsedSource> {
    if !path.is_file() {
        return Err("The selected import source is not a readable file.".to_string());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let bytes = fs::read(path).map_err(|error| format!("Could not read import source: {error}"))?;
    let fingerprint = format!("sha256-{}", hex_digest(&bytes));
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("import")
        .to_string();

    match extension.as_str() {
        "csv" => parse_csv(&bytes, filename, fingerprint),
        "xlsx" | "xls" => parse_workbook(path, &extension, filename, fingerprint),
        _ => Err("Import source must be a .csv, .xlsx, or .xls file.".to_string()),
    }
}

fn parse_csv(bytes: &[u8], filename: String, fingerprint: String) -> CommandResult<ParsedSource> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(bytes);
    let mut records = reader.records();
    let header = records
        .next()
        .ok_or_else(|| "CSV import source is empty.".to_string())?
        .map_err(|error| format!("Could not parse CSV header: {error}"))?;
    let mut headers = header.iter().map(ToString::to_string).collect::<Vec<_>>();
    if headers.iter().all(|header| header.trim().is_empty()) {
        return Err("CSV import source has no usable header row.".to_string());
    }

    let mut rows = Vec::new();
    for (index, record) in records.enumerate() {
        let record = record.map_err(|error| format!("Could not parse CSV row: {error}"))?;
        let source_row = record
            .position()
            .map(|position| position.line())
            .unwrap_or(index as u64 + 2);
        let values = record.iter().map(ToString::to_string).collect::<Vec<_>>();
        rows.push(ParsedRow { source_row, values });
    }
    for source_row in blank_csv_record_lines(bytes) {
        if source_row > 1
            && rows.iter().all(|row| {
                row.source_row != source_row
                    || row.values.iter().any(|value| !value.trim().is_empty())
            })
        {
            rows.push(ParsedRow {
                source_row,
                values: vec![String::new(); headers.len()],
            });
        }
    }
    rows.sort_by_key(|row| {
        (
            row.source_row,
            row.values.iter().any(|value| !value.trim().is_empty()),
        )
    });
    let unheaded_columns = normalize_unheaded_columns(&mut headers, &mut rows);

    Ok(ParsedSource {
        filename,
        selected_sheet: "CSV".to_string(),
        fingerprint,
        headers,
        unheaded_columns,
        rows,
    })
}

fn blank_csv_record_lines(bytes: &[u8]) -> Vec<u64> {
    let mut blank_lines = Vec::new();
    let mut line = 1u64;
    let mut record_start_line = line;
    let mut record_has_content = false;
    let mut in_quotes = false;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                record_has_content = true;
                if in_quotes && bytes.get(index + 1) == Some(&b'"') {
                    index += 1;
                } else {
                    in_quotes = !in_quotes;
                }
            }
            b'\n' if !in_quotes => {
                if !record_has_content {
                    blank_lines.push(record_start_line);
                }
                line += 1;
                record_start_line = line;
                record_has_content = false;
            }
            b'\n' => line += 1,
            b'\r' => {}
            _ => record_has_content = true,
        }
        index += 1;
    }
    if !record_has_content && !bytes.ends_with(b"\n") {
        blank_lines.push(record_start_line);
    }
    blank_lines
}

fn parse_workbook(
    path: &Path,
    extension: &str,
    filename: String,
    fingerprint: String,
) -> CommandResult<ParsedSource> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|error| format!("Could not open .{extension} workbook import source: {error}"))?;
    let sheet_names = workbook
        .sheets_metadata()
        .iter()
        .filter(|sheet| sheet.visible == SheetVisible::Visible)
        .map(|sheet| sheet.name.clone())
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for sheet_name in sheet_names {
        let range = workbook.worksheet_range(&sheet_name).map_err(|error| {
            format!("Could not read .{extension} workbook sheet '{sheet_name}': {error}")
        })?;
        if range.is_empty() {
            continue;
        }
        let mut sheet_rows = range.rows();
        let Some(header_row) = sheet_rows.next() else {
            continue;
        };
        let mut headers = header_row.iter().map(cell_to_string).collect::<Vec<_>>();
        if headers.iter().all(|header| header.trim().is_empty()) {
            continue;
        }
        let mut rows = Vec::new();
        for (index, row) in sheet_rows.enumerate() {
            let mut values = row.iter().map(cell_to_string).collect::<Vec<_>>();
            values.resize(headers.len(), String::new());
            rows.push(ParsedRow {
                source_row: index as u64 + 2,
                values,
            });
        }
        let unheaded_columns = normalize_unheaded_columns(&mut headers, &mut rows);
        candidates.push(ParsedSource {
            filename: filename.clone(),
            selected_sheet: sheet_name,
            fingerprint: fingerprint.clone(),
            headers,
            unheaded_columns,
            rows,
        });
    }

    let inventory_candidates = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            candidate
                .selected_sheet
                .eq_ignore_ascii_case("Inventory")
                .then_some(index)
        })
        .collect::<Vec<_>>();
    if let [index] = inventory_candidates.as_slice() {
        return Ok(candidates.remove(*index));
    }

    match candidates.len() {
        0 => Err(format!(
            ".{extension} workbook has no visible nonempty sheet with a header row."
        )),
        1 => Ok(candidates.pop().expect("one workbook candidate")),
        _ => {
            let sheet_names = candidates
                .iter()
                .map(|candidate| candidate.selected_sheet.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                ".{extension} workbook has multiple visible usable sheets and no single sheet named Inventory: {sheet_names}."
            ))
        }
    }
}

fn normalize_unheaded_columns(headers: &mut Vec<String>, rows: &mut [ParsedRow]) -> Vec<usize> {
    let declared_width = headers.len();
    let meaningful_width = rows
        .iter()
        .filter_map(|row| {
            row.values
                .iter()
                .rposition(|value| !value.trim().is_empty())
                .map(|index| index + 1)
        })
        .max()
        .unwrap_or_default();
    let retained_width = declared_width.max(meaningful_width);
    headers.resize(retained_width, String::new());

    let mut unheaded_columns = Vec::new();
    for (index, header) in headers.iter_mut().enumerate() {
        let has_nonblank_value = rows.iter().any(|row| {
            row.values
                .get(index)
                .is_some_and(|value| !value.trim().is_empty())
        });
        if header.trim().is_empty()
            && (has_nonblank_value || (index >= declared_width && index < meaningful_width))
        {
            *header = format!("__unheaded_column_{}", index + 1);
            unheaded_columns.push(index);
        }
    }
    for row in rows {
        row.values.resize(retained_width, String::new());
    }
    unheaded_columns
}

fn cell_to_string(cell: &Data) -> String {
    if let Some(date) = cell.as_date() {
        return date.format("%Y-%m-%d").to_string();
    }
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::Error(error) => format!("#EXCEL_ERROR:{error:?}"),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
    }
}

pub(super) fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
