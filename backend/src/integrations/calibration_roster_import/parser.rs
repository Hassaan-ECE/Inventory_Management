use std::{fs, path::Path};

use calamine::{open_workbook_auto, Data, DataType, Reader, SheetVisible};
use sha2::{Digest, Sha256};

use crate::model::CommandResult;

#[derive(Debug, Clone)]
pub(super) struct ParsedRosterSource {
    pub filename: String,
    pub fingerprint: String,
    pub contributing_sheets: Vec<String>,
    pub rows: Vec<ParsedRosterRow>,
}

#[derive(Debug, Clone)]
pub(super) struct ParsedRosterRow {
    pub source_sheet: String,
    pub source_row: u64,
    pub fields: ParsedRosterFields,
}

#[derive(Debug, Clone, Default)]
pub(super) struct ParsedRosterFields {
    pub asset_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub last_calibrated_at: Option<String>,
    pub calibration_due_at: Option<String>,
    pub status: Option<String>,
    pub calibration_vendor: Option<String>,
    pub location: Option<String>,
    pub assigned_to: Option<String>,
    pub condition: Option<String>,
    pub comments: Option<String>,
    pub certificate_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HeaderTarget {
    AssetNumber,
    SerialNumber,
    Manufacturer,
    Model,
    Description,
    LastCalibratedAt,
    CalibrationDueAt,
    Status,
    CalibrationVendor,
    Location,
    AssignedTo,
    Condition,
    Comments,
    CertificateRef,
}

type HeaderMapping = Vec<(String, Option<HeaderTarget>)>;
type HeaderRow = (usize, HeaderMapping);

pub(super) fn parse_roster_source(path: &Path) -> CommandResult<ParsedRosterSource> {
    if !path.is_file() {
        return Err("The selected calibration roster is not a readable file.".to_string());
    }
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("xlsx"))
    {
        return Err("Calibration roster source must be an .xlsx workbook.".to_string());
    }

    let bytes = fs::read(path)
        .map_err(|error| format!("Could not read calibration roster source: {error}"))?;
    let fingerprint = format!("sha256-{}", hex_digest(&bytes));
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("calibration-roster.xlsx")
        .to_string();
    let mut workbook = open_workbook_auto(path)
        .map_err(|error| format!("Could not open calibration roster workbook: {error}"))?;
    let visible_sheets = workbook
        .sheets_metadata()
        .iter()
        .filter(|sheet| sheet.visible == SheetVisible::Visible)
        .map(|sheet| sheet.name.clone())
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    let mut contributing_sheets = Vec::new();
    let mut rejected_sheets = Vec::new();
    for sheet_name in visible_sheets {
        let range = workbook.worksheet_range(&sheet_name).map_err(|error| {
            format!("Could not read calibration roster sheet '{sheet_name}': {error}")
        })?;
        if range.is_empty() {
            continue;
        }
        let row_offset = range.start().map(|(row, _)| row as u64).unwrap_or(0);
        let sheet_rows = range.rows().collect::<Vec<_>>();
        let Some((header_index, headers)) = find_header_row(&sheet_rows) else {
            rejected_sheets.push(sheet_name);
            continue;
        };

        contributing_sheets.push(sheet_name.clone());
        for (index, row) in sheet_rows.iter().enumerate().skip(header_index + 1) {
            if row
                .iter()
                .all(|cell| cell_to_string(cell).trim().is_empty())
            {
                continue;
            }
            let source_row = row_offset + index as u64 + 1;
            rows.push(build_row(&sheet_name, source_row, &headers, row));
        }
    }

    if contributing_sheets.is_empty() {
        let suffix = if rejected_sheets.is_empty() {
            String::new()
        } else {
            format!(
                " Visible sheets without a recognized roster header: {}.",
                rejected_sheets.join(", ")
            )
        };
        return Err(format!(
            "Calibration roster workbook has no visible sheet with recognized equipment and calibration headers.{suffix}"
        ));
    }

    Ok(ParsedRosterSource {
        filename,
        fingerprint,
        contributing_sheets,
        rows,
    })
}

fn find_header_row(rows: &[&[Data]]) -> Option<HeaderRow> {
    rows.iter().enumerate().find_map(|(index, row)| {
        let headers = row
            .iter()
            .enumerate()
            .map(|(column, cell)| {
                let raw = cell_to_string(cell);
                let label = if raw.trim().is_empty() {
                    format!("Column {}", column + 1)
                } else {
                    raw.trim().to_string()
                };
                (label, map_header(&raw))
            })
            .collect::<Vec<_>>();
        let targets = headers
            .iter()
            .filter_map(|(_, target)| *target)
            .collect::<std::collections::BTreeSet<_>>();
        let identity_count = [
            HeaderTarget::AssetNumber,
            HeaderTarget::SerialNumber,
            HeaderTarget::Description,
        ]
        .into_iter()
        .filter(|target| targets.contains(target))
        .count();
        let has_calibration = [
            HeaderTarget::LastCalibratedAt,
            HeaderTarget::CalibrationDueAt,
            HeaderTarget::Status,
            HeaderTarget::CalibrationVendor,
        ]
        .into_iter()
        .any(|target| targets.contains(&target));
        (identity_count >= 2 && targets.contains(&HeaderTarget::Description) && has_calibration)
            .then_some((index, headers))
    })
}

fn build_row(
    sheet_name: &str,
    source_row: u64,
    headers: &HeaderMapping,
    row: &[Data],
) -> ParsedRosterRow {
    let mut fields = ParsedRosterFields::default();
    for (index, (header, target)) in headers.iter().enumerate() {
        let value = row.get(index).map(cell_to_string).unwrap_or_default();
        let _ = header;
        if let Some(target) = target {
            assign_field(&mut fields, *target, value);
        }
    }
    ParsedRosterRow {
        source_sheet: sheet_name.to_string(),
        source_row,
        fields,
    }
}

fn assign_field(fields: &mut ParsedRosterFields, target: HeaderTarget, value: String) {
    let value = optional_text(value);
    let slot = match target {
        HeaderTarget::AssetNumber => &mut fields.asset_number,
        HeaderTarget::SerialNumber => &mut fields.serial_number,
        HeaderTarget::Manufacturer => &mut fields.manufacturer,
        HeaderTarget::Model => &mut fields.model,
        HeaderTarget::Description => &mut fields.description,
        HeaderTarget::LastCalibratedAt => &mut fields.last_calibrated_at,
        HeaderTarget::CalibrationDueAt => &mut fields.calibration_due_at,
        HeaderTarget::Status => &mut fields.status,
        HeaderTarget::CalibrationVendor => &mut fields.calibration_vendor,
        HeaderTarget::Location => &mut fields.location,
        HeaderTarget::AssignedTo => &mut fields.assigned_to,
        HeaderTarget::Condition => &mut fields.condition,
        HeaderTarget::Comments => &mut fields.comments,
        HeaderTarget::CertificateRef => &mut fields.certificate_ref,
    };
    if slot.is_none() && value.is_some() {
        *slot = value;
    }
}

fn map_header(value: &str) -> Option<HeaderTarget> {
    match normalized_header(value).as_str() {
        "asset" | "assetno" | "assetnumber" | "assetnumbers" => Some(HeaderTarget::AssetNumber),
        "serial" | "serialno" | "serialnumber" | "serialnumbers" => {
            Some(HeaderTarget::SerialNumber)
        }
        "manufacturer" | "mfg" => Some(HeaderTarget::Manufacturer),
        "model" | "modelnumber" => Some(HeaderTarget::Model),
        "description" | "equipmentdescription" => Some(HeaderTarget::Description),
        "newcaldate" | "lastcaldate" | "lastcalibrateddate" | "lastcalibrationdate" => {
            Some(HeaderTarget::LastCalibratedAt)
        }
        "calduedate" | "calibrationdue" | "calibrationduedate" => {
            Some(HeaderTarget::CalibrationDueAt)
        }
        "calibrationstatus" | "calstatus" | "status" => Some(HeaderTarget::Status),
        "calibratedby" | "calibrationvendor" | "calvendor" | "lastcalibratedby" => {
            Some(HeaderTarget::CalibrationVendor)
        }
        "location" => Some(HeaderTarget::Location),
        "assignedto" | "owner" => Some(HeaderTarget::AssignedTo),
        "condition" => Some(HeaderTarget::Condition),
        "comment" | "comments" | "notes" => Some(HeaderTarget::Comments),
        "certificate" | "certificateref" | "certificatereference" => {
            Some(HeaderTarget::CertificateRef)
        }
        _ => None,
    }
}

fn normalized_header(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn optional_text(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
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
