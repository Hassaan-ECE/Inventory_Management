use std::{collections::BTreeMap, path::Path};

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};

use crate::modules::te_lab_components::{
    catalog_model::{
        grid_coordinate_label, summarize_part_stock, Part, PartStockSummary, StockPlacement,
        StockStatus, StorageArea, StorageContainer,
    },
    model::CommandResult,
};

use super::ExcelExportStats;

const PARTS_SHEET: &str = "Parts";
const ARCHIVED_PARTS_SHEET: &str = "Archived Parts";
const PLACEMENTS_SHEET: &str = "Stock Placements";
const STORAGE_SHEET: &str = "Storage Layout";
const ATTRIBUTES_SHEET: &str = "Attributes";
const LEGACY_FIELDS_SHEET: &str = "Legacy Fields";

const PART_HEADERS: &[&str] = &[
    "Part UUID",
    "Database ID",
    "Internal Part Number",
    "Category",
    "Subcategory",
    "Manufacturer",
    "Manufacturer Part Number",
    "Value / Label",
    "Mounting Type",
    "Package Type",
    "Description",
    "Default Unit",
    "Derived Totals",
    "Stock Status",
    "Reorder Point",
    "Target Quantity",
    "Part Status",
    "Supplier",
    "Supplier SKU",
    "Supplier Packaging",
    "Product URL",
    "Datasheet URL",
    "Picture Path",
    "Notes",
    "Archived",
    "Created At",
    "Updated At",
];

const PLACEMENT_HEADERS: &[&str] = &[
    "Placement UUID",
    "Part UUID",
    "Internal Part Number",
    "Manufacturer",
    "Manufacturer Part Number",
    "Value / Label",
    "Area UUID",
    "Area",
    "Container UUID",
    "Container",
    "Coordinate",
    "Full Location Path",
    "Quantity",
    "Unit",
    "Packaging",
    "Lot Code",
    "Date Code",
    "Condition",
    "Count State",
    "Last Counted At",
    "Last Counted By",
    "Placement Notes",
    "Archived",
    "Created At",
    "Updated At",
];

const STORAGE_HEADERS: &[&str] = &[
    "Area UUID",
    "Area Name",
    "Area Type",
    "Area Owner",
    "Area Description",
    "Area Archived",
    "Container UUID",
    "Container Name",
    "Container Type",
    "Grid Enabled",
    "Row Count",
    "Column Count",
    "Row Start",
    "Origin",
    "Container Description",
    "Container Archived",
    "Area Created At",
    "Area Updated At",
    "Container Created At",
    "Container Updated At",
];

const ATTRIBUTE_HEADERS: &[&str] = &[
    "Part UUID",
    "Internal Part Number",
    "Manufacturer Part Number",
    "Category",
    "Attribute Key",
    "Attribute Value",
    "Attribute Unit",
];

const LEGACY_HEADERS: &[&str] = &[
    "Part UUID",
    "Internal Part Number",
    "Serial Number",
    "Project Name",
    "Assigned To",
    "Lifecycle Status",
    "Working Status",
    "Condition",
    "Verified in Survey",
    "Manual Entry",
];

pub(crate) fn write_catalog_workbook(
    parts: &[Part],
    areas: &[StorageArea],
    containers: &[StorageContainer],
    placements: &[StockPlacement],
    output_path: impl AsRef<Path>,
) -> CommandResult<ExcelExportStats> {
    let output_path = output_path.as_ref();
    let formats = CatalogFormats::new();
    let mut workbook = Workbook::new();
    let summaries = parts
        .iter()
        .map(|part| {
            (
                part.entry_uuid.clone(),
                summarize_part_stock(part, placements),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut active_parts = parts
        .iter()
        .filter(|part| !part.archived)
        .collect::<Vec<_>>();
    let mut archived_parts = parts
        .iter()
        .filter(|part| part.archived)
        .collect::<Vec<_>>();
    active_parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));
    archived_parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));

    build_sheet(
        workbook.add_worksheet(),
        PARTS_SHEET,
        PART_HEADERS,
        active_parts
            .into_iter()
            .map(|part| part_row(part, summaries.get(&part.entry_uuid)))
            .collect(),
        &formats,
    )
    .map_err(export_error)?;

    build_sheet(
        workbook.add_worksheet(),
        ARCHIVED_PARTS_SHEET,
        PART_HEADERS,
        archived_parts
            .into_iter()
            .map(|part| part_row(part, summaries.get(&part.entry_uuid)))
            .collect(),
        &formats,
    )
    .map_err(export_error)?;

    build_sheet(
        workbook.add_worksheet(),
        PLACEMENTS_SHEET,
        PLACEMENT_HEADERS,
        placement_rows(parts, areas, containers, placements),
        &formats,
    )
    .map_err(export_error)?;

    build_sheet(
        workbook.add_worksheet(),
        STORAGE_SHEET,
        STORAGE_HEADERS,
        storage_rows(areas, containers),
        &formats,
    )
    .map_err(export_error)?;

    build_sheet(
        workbook.add_worksheet(),
        ATTRIBUTES_SHEET,
        ATTRIBUTE_HEADERS,
        attribute_rows(parts),
        &formats,
    )
    .map_err(export_error)?;

    build_sheet(
        workbook.add_worksheet(),
        LEGACY_FIELDS_SHEET,
        LEGACY_HEADERS,
        legacy_rows(parts),
        &formats,
    )
    .map_err(export_error)?;

    workbook.save(output_path).map_err(export_error)?;

    let archived_count = parts.iter().filter(|part| part.archived).count();
    Ok(ExcelExportStats {
        archived_count,
        inventory_count: parts.len().saturating_sub(archived_count),
        output_path: output_path.to_string_lossy().to_string(),
        total_count: parts.len(),
    })
}

#[cfg(test)]
pub(super) fn catalog_sheet_headers() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (PARTS_SHEET, PART_HEADERS.to_vec()),
        (ARCHIVED_PARTS_SHEET, PART_HEADERS.to_vec()),
        (PLACEMENTS_SHEET, PLACEMENT_HEADERS.to_vec()),
        (STORAGE_SHEET, STORAGE_HEADERS.to_vec()),
        (ATTRIBUTES_SHEET, ATTRIBUTE_HEADERS.to_vec()),
        (LEGACY_FIELDS_SHEET, LEGACY_HEADERS.to_vec()),
    ]
}

fn part_row(part: &Part, summary: Option<&PartStockSummary>) -> Vec<CellValue> {
    vec![
        part.entry_uuid.clone().into(),
        part.database_id.map(|value| value as f64).into(),
        part.internal_part_number.clone().into(),
        part.category.clone().into(),
        part.subcategory.clone().into(),
        part.manufacturer.clone().into(),
        part.manufacturer_part_number.clone().into(),
        part.display_value.clone().into(),
        part.mounting_type.clone().into(),
        part.package_type.clone().into(),
        part.description.clone().into(),
        part.default_unit_of_measure.clone().into(),
        summary.map(format_totals).unwrap_or_default().into(),
        summary
            .map(|summary| stock_status_text(&summary.stock_status))
            .unwrap_or("no_stock")
            .into(),
        part.reorder_point.into(),
        part.target_quantity.into(),
        part.part_status.clone().into(),
        part.supplier.clone().into(),
        part.supplier_sku.clone().into(),
        part.supplier_packaging.clone().into(),
        part.product_url.clone().into(),
        part.datasheet_url.clone().into(),
        part.picture_path.clone().into(),
        part.notes.clone().into(),
        yes_if(part.archived).into(),
        part.created_at.clone().into(),
        part.updated_at.clone().into(),
    ]
}

fn placement_rows(
    parts: &[Part],
    areas: &[StorageArea],
    containers: &[StorageContainer],
    placements: &[StockPlacement],
) -> Vec<Vec<CellValue>> {
    let parts_by_id = parts
        .iter()
        .map(|part| (part.entry_uuid.as_str(), part))
        .collect::<BTreeMap<_, _>>();
    let areas_by_id = areas
        .iter()
        .map(|area| (area.area_uuid.as_str(), area))
        .collect::<BTreeMap<_, _>>();
    let containers_by_id = containers
        .iter()
        .map(|container| (container.container_uuid.as_str(), container))
        .collect::<BTreeMap<_, _>>();
    let mut placements = placements.iter().collect::<Vec<_>>();
    placements.sort_by(|left, right| left.placement_uuid.cmp(&right.placement_uuid));

    placements
        .into_iter()
        .map(|placement| {
            let part = parts_by_id.get(placement.part_uuid.as_str()).copied();
            let container = containers_by_id
                .get(placement.container_uuid.as_str())
                .copied();
            let area = container
                .and_then(|container| areas_by_id.get(container.area_uuid.as_str()).copied());
            let coordinate = placement_coordinate(placement, container);
            let full_path = [
                area.map(|area| area.name.as_str())
                    .unwrap_or("Unknown area"),
                container
                    .map(|container| container.name.as_str())
                    .unwrap_or("Unknown container"),
                coordinate.as_str(),
            ]
            .join(" / ");
            vec![
                placement.placement_uuid.clone().into(),
                placement.part_uuid.clone().into(),
                part.map(|part| part.internal_part_number.clone())
                    .unwrap_or_default()
                    .into(),
                part.map(|part| part.manufacturer.clone())
                    .unwrap_or_default()
                    .into(),
                part.map(|part| part.manufacturer_part_number.clone())
                    .unwrap_or_default()
                    .into(),
                part.map(|part| part.display_value.clone())
                    .unwrap_or_default()
                    .into(),
                area.map(|area| area.area_uuid.clone())
                    .unwrap_or_default()
                    .into(),
                area.map(|area| area.name.clone())
                    .unwrap_or_else(|| "Unknown area".to_string())
                    .into(),
                placement.container_uuid.clone().into(),
                container
                    .map(|container| container.name.clone())
                    .unwrap_or_else(|| "Unknown container".to_string())
                    .into(),
                coordinate.into(),
                full_path.into(),
                CellValue::Number(placement.quantity),
                placement.unit_of_measure.clone().into(),
                placement.packaging.clone().into(),
                placement.lot_code.clone().into(),
                placement.date_code.clone().into(),
                placement.condition.clone().into(),
                placement.count_state.clone().into(),
                placement.last_counted_at.clone().unwrap_or_default().into(),
                placement.last_counted_by.clone().into(),
                placement.notes.clone().into(),
                yes_if(placement.archived).into(),
                placement.created_at.clone().into(),
                placement.updated_at.clone().into(),
            ]
        })
        .collect()
}

fn storage_rows(areas: &[StorageArea], containers: &[StorageContainer]) -> Vec<Vec<CellValue>> {
    let mut areas = areas.iter().collect::<Vec<_>>();
    areas.sort_by(|left, right| left.area_uuid.cmp(&right.area_uuid));
    let mut rows = Vec::new();
    for area in areas {
        let mut area_containers = containers
            .iter()
            .filter(|container| container.area_uuid == area.area_uuid)
            .collect::<Vec<_>>();
        area_containers.sort_by(|left, right| left.container_uuid.cmp(&right.container_uuid));
        if area_containers.is_empty() {
            rows.push(storage_row(area, None));
        } else {
            rows.extend(
                area_containers
                    .into_iter()
                    .map(|container| storage_row(area, Some(container))),
            );
        }
    }
    rows
}

fn storage_row(area: &StorageArea, container: Option<&StorageContainer>) -> Vec<CellValue> {
    vec![
        area.area_uuid.clone().into(),
        area.name.clone().into(),
        area.area_type.clone().into(),
        area.owner.clone().into(),
        area.description.clone().into(),
        yes_if(area.archived).into(),
        container
            .map(|container| container.container_uuid.clone())
            .unwrap_or_default()
            .into(),
        container
            .map(|container| container.name.clone())
            .unwrap_or_default()
            .into(),
        container
            .map(|container| container.container_type.clone())
            .unwrap_or_default()
            .into(),
        container
            .map(|container| yes_if(container.grid_enabled))
            .unwrap_or_default()
            .into(),
        container
            .and_then(|container| container.row_count.map(f64::from))
            .into(),
        container
            .and_then(|container| container.column_count.map(f64::from))
            .into(),
        container
            .map(|container| f64::from(container.row_start))
            .into(),
        container
            .map(|container| container.origin.clone())
            .unwrap_or_default()
            .into(),
        container
            .map(|container| container.description.clone())
            .unwrap_or_default()
            .into(),
        container
            .map(|container| yes_if(container.archived))
            .unwrap_or_default()
            .into(),
        area.created_at.clone().into(),
        area.updated_at.clone().into(),
        container
            .map(|container| container.created_at.clone())
            .unwrap_or_default()
            .into(),
        container
            .map(|container| container.updated_at.clone())
            .unwrap_or_default()
            .into(),
    ]
}

fn attribute_rows(parts: &[Part]) -> Vec<Vec<CellValue>> {
    let mut parts = parts.iter().collect::<Vec<_>>();
    parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));
    parts
        .into_iter()
        .flat_map(|part| {
            part.attributes.iter().map(move |(key, value)| {
                vec![
                    part.entry_uuid.clone().into(),
                    part.internal_part_number.clone().into(),
                    part.manufacturer_part_number.clone().into(),
                    part.category.clone().into(),
                    key.clone().into(),
                    value.value.clone().into(),
                    value.unit.clone().into(),
                ]
            })
        })
        .collect()
}

fn legacy_rows(parts: &[Part]) -> Vec<Vec<CellValue>> {
    let mut parts = parts.iter().collect::<Vec<_>>();
    parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));
    parts
        .into_iter()
        .map(|part| {
            vec![
                part.entry_uuid.clone().into(),
                part.internal_part_number.clone().into(),
                part.legacy.serial_number.clone().into(),
                part.legacy.project_name.clone().into(),
                part.legacy.assigned_to.clone().into(),
                part.legacy.lifecycle_status.clone().into(),
                part.legacy.working_status.clone().into(),
                part.legacy.condition.clone().into(),
                yes_if(part.legacy.verified_in_survey).into(),
                yes_if(part.legacy.manual_entry).into(),
            ]
        })
        .collect()
}

fn build_sheet(
    worksheet: &mut Worksheet,
    name: &str,
    headers: &[&str],
    rows: Vec<Vec<CellValue>>,
    formats: &CatalogFormats,
) -> Result<(), XlsxError> {
    worksheet.set_name(name)?;
    worksheet.set_landscape();
    worksheet.set_print_fit_to_pages(1, 0);
    worksheet.set_freeze_panes(1, 0)?;
    worksheet.set_row_height(0, 30.0)?;

    for (column_index, header) in headers.iter().enumerate() {
        let column_index = column_index as u16;
        worksheet.set_column_width(column_index, column_width(header))?;
        worksheet.write_string_with_format(0, column_index, *header, &formats.header)?;
    }

    for (row_index, row) in rows.iter().enumerate() {
        let worksheet_row = row_index as u32 + 1;
        worksheet.set_row_height(worksheet_row, 24.0)?;
        for (column_index, value) in row.iter().enumerate() {
            let column_index = column_index as u16;
            match value {
                CellValue::Blank => {
                    worksheet.write_string_with_format(
                        worksheet_row,
                        column_index,
                        "",
                        formats.text(row_index),
                    )?;
                }
                CellValue::Number(value) => {
                    worksheet.write_number_with_format(
                        worksheet_row,
                        column_index,
                        *value,
                        formats.number(row_index),
                    )?;
                }
                CellValue::Text(value) => {
                    worksheet.write_string_with_format(
                        worksheet_row,
                        column_index,
                        value,
                        formats.text(row_index),
                    )?;
                }
            }
        }
    }

    worksheet.autofilter(0, 0, rows.len() as u32, (headers.len() - 1) as u16)?;
    Ok(())
}

fn placement_coordinate(
    placement: &StockPlacement,
    container: Option<&StorageContainer>,
) -> String {
    match (
        container.filter(|container| container.grid_enabled),
        placement.row_index,
        placement.column_index,
    ) {
        (Some(container), Some(row_index), Some(column_index)) => {
            grid_coordinate_label(container, row_index, column_index)
        }
        _ if !placement.freeform_position.is_empty() => placement.freeform_position.clone(),
        _ => "Unassigned".to_string(),
    }
}

fn format_totals(summary: &PartStockSummary) -> String {
    summary
        .totals
        .iter()
        .map(|total| {
            format!(
                "{} {}",
                compact_number(total.quantity),
                total.unit_of_measure
            )
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn compact_number(value: f64) -> String {
    if value.fract().abs() <= f64::EPSILON {
        format!("{value:.0}")
    } else {
        let value = format!("{value:.4}");
        value
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn stock_status_text(status: &StockStatus) -> &'static str {
    match status {
        StockStatus::Archived => "archived",
        StockStatus::NoStock => "no_stock",
        StockStatus::UnitReview => "unit_review",
        StockStatus::LowStock => "low_stock",
        StockStatus::InStock => "in_stock",
        StockStatus::MixedUnits => "mixed_units",
    }
}

fn yes_if(value: bool) -> &'static str {
    if value {
        "Yes"
    } else {
        ""
    }
}

fn column_width(header: &str) -> f64 {
    match header {
        "Description" | "Notes" | "Placement Notes" | "Full Location Path" => 34.0,
        "Product URL" | "Datasheet URL" | "Picture Path" => 38.0,
        "Manufacturer Part Number" | "Internal Part Number" => 24.0,
        "Created At" | "Updated At" | "Last Counted At" => 24.0,
        _ => header.chars().count().clamp(12, 22) as f64,
    }
}

fn export_error(error: impl std::fmt::Display) -> String {
    format!("Excel export failed: {error}")
}

enum CellValue {
    Blank,
    Number(f64),
    Text(String),
}

impl From<String> for CellValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for CellValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<Option<f64>> for CellValue {
    fn from(value: Option<f64>) -> Self {
        value.map(Self::Number).unwrap_or(Self::Blank)
    }
}

struct CatalogFormats {
    header: Format,
    number_even: Format,
    number_odd: Format,
    text_even: Format,
    text_odd: Format,
}

impl CatalogFormats {
    fn new() -> Self {
        Self {
            header: Format::new()
                .set_bold()
                .set_font_color("FFFFFF")
                .set_background_color("1F2937")
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_text_wrap()
                .set_border(FormatBorder::Thin)
                .set_border_color("374151")
                .set_border_bottom(FormatBorder::Medium),
            number_even: cell_format("F9FAFB", FormatAlign::Right).set_num_format("0.####"),
            number_odd: cell_format("F3F4F6", FormatAlign::Right).set_num_format("0.####"),
            text_even: cell_format("F9FAFB", FormatAlign::Left).set_text_wrap(),
            text_odd: cell_format("F3F4F6", FormatAlign::Left).set_text_wrap(),
        }
    }

    fn number(&self, row_index: usize) -> &Format {
        if row_index.is_multiple_of(2) {
            &self.number_even
        } else {
            &self.number_odd
        }
    }

    fn text(&self, row_index: usize) -> &Format {
        if row_index.is_multiple_of(2) {
            &self.text_even
        } else {
            &self.text_odd
        }
    }
}

fn cell_format(background: &'static str, align: FormatAlign) -> Format {
    Format::new()
        .set_font_color("1F2937")
        .set_font_size(10)
        .set_background_color(background)
        .set_align(align)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_border_color("D1D5DB")
}
