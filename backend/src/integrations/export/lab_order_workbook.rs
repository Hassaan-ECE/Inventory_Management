use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    path::Path,
};

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};
use serde::Deserialize;
use url::Url;

use crate::modules::te_lab_components::{
    catalog_model::{
        grid_coordinate_label, summarize_part_stock, Part, StockPlacement, StockStatus,
        StorageContainer,
    },
    model::CommandResult,
    store::InventoryDb as LabInventoryDb,
};

use super::ExcelExportStats;

const ORDER_SHEET: &str = "Order Request";

pub(crate) const ORDER_HEADERS: [&str; 12] = [
    "Component Type",
    "Value",
    "Manufacturer",
    "MPN",
    "Supplier",
    "Supplier SKU",
    "Product Link",
    "Stock Status",
    "Current Qty",
    "Qty Requested",
    "Location",
    "Notes",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LabOrderRequestLineInput {
    pub part_uuid: String,
    pub requested_quantity: u32,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LabOrderRow {
    pub component_type: String,
    pub value: String,
    pub manufacturer: String,
    pub mpn: String,
    pub supplier: String,
    pub supplier_sku: String,
    pub product_link: String,
    pub stock_status: String,
    pub current_quantity: u32,
    pub requested_quantity: u32,
    pub location: String,
    pub note: String,
}

pub(crate) fn build_lab_order_rows(
    db: &LabInventoryDb,
    input: &[LabOrderRequestLineInput],
) -> CommandResult<Vec<LabOrderRow>> {
    if input.is_empty() {
        return Err("Select at least one component to export for order.".to_string());
    }

    let mut seen = HashSet::new();
    let mut prepared = Vec::with_capacity(input.len());
    for line in input {
        let part_uuid = line.part_uuid.trim().to_string();
        if part_uuid.is_empty() {
            return Err("Each order line must include a part UUID.".to_string());
        }
        if !seen.insert(part_uuid.clone()) {
            return Err(format!(
                "Rejected duplicate part selection for order export: {part_uuid}."
            ));
        }
        if line.requested_quantity == 0 {
            return Err(format!(
                "Requested quantity must be greater than zero for part {part_uuid}."
            ));
        }
        prepared.push((
            part_uuid,
            line.requested_quantity,
            line.note.trim().to_string(),
        ));
    }

    let containers = db
        .load_storage_containers()?
        .into_iter()
        .map(|container| (container.container_uuid.clone(), container))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(prepared.len());
    for (part_uuid, requested_quantity, note) in prepared {
        let part = db.find_part(&part_uuid)?.ok_or_else(|| {
            format!("Selected part was not found in the Lab catalog: {part_uuid}.")
        })?;
        if part.archived {
            return Err(format!(
                "Cannot export archived part for order: {}.",
                display_part_label(&part)
            ));
        }

        let placements = db.load_stock_placements_for_part(&part.entry_uuid)?;
        let active_placements = placements
            .iter()
            .filter(|placement| !placement.archived)
            .cloned()
            .collect::<Vec<_>>();
        let summary = summarize_part_stock(&part, &active_placements);
        let current_quantity = pcs_current_quantity(&part, &summary.totals)?;
        let location = format_locations(&active_placements, &containers);

        rows.push(LabOrderRow {
            component_type: part.subcategory.clone(),
            value: part.display_value.clone(),
            manufacturer: part.manufacturer.clone(),
            mpn: part.manufacturer_part_number.clone(),
            supplier: part.supplier.clone(),
            supplier_sku: part.supplier_sku.clone(),
            product_link: part.product_url.clone(),
            stock_status: stock_status_text(&summary.stock_status).to_string(),
            current_quantity,
            requested_quantity,
            location,
            note,
        });
    }

    rows.sort_by(|left, right| {
        left.component_type
            .cmp(&right.component_type)
            .then_with(|| left.value.cmp(&right.value))
            .then_with(|| left.mpn.cmp(&right.mpn))
    });
    Ok(rows)
}

pub(crate) fn write_lab_order_workbook(
    rows: &[LabOrderRow],
    output_path: impl AsRef<Path>,
) -> CommandResult<ExcelExportStats> {
    let output_path = output_path.as_ref();
    let formats = OrderFormats::new();
    let mut workbook = Workbook::new();
    build_order_sheet(workbook.add_worksheet(), rows, &formats).map_err(export_error)?;
    workbook.save(output_path).map_err(export_error)?;

    Ok(ExcelExportStats {
        archived_count: 0,
        inventory_count: rows.len(),
        output_path: output_path.to_string_lossy().to_string(),
        total_count: rows.len(),
    })
}

fn pcs_current_quantity(
    part: &Part,
    totals: &[crate::modules::te_lab_components::catalog_model::QuantityTotal],
) -> CommandResult<u32> {
    if totals.is_empty() {
        return Ok(0);
    }
    if totals.len() != 1 || totals[0].unit_of_measure != "pcs" {
        return Err(format!(
            "Order export requires whole pieces only for {}.",
            display_part_label(part)
        ));
    }
    whole_piece_quantity(totals[0].quantity, part)
}

fn whole_piece_quantity(quantity: f64, part: &Part) -> CommandResult<u32> {
    if !quantity.is_finite() || quantity < 0.0 {
        return Err(format!(
            "Order export requires whole pieces only for {}.",
            display_part_label(part)
        ));
    }
    if (quantity - quantity.floor()).abs() > f64::EPSILON {
        return Err(format!(
            "Order export requires whole pieces only for {}.",
            display_part_label(part)
        ));
    }
    if quantity > f64::from(u32::MAX) {
        return Err(format!(
            "Current quantity is too large for order export: {}.",
            display_part_label(part)
        ));
    }
    Ok(quantity as u32)
}

fn format_locations(
    placements: &[StockPlacement],
    containers: &BTreeMap<String, StorageContainer>,
) -> String {
    let mut labels = BTreeSet::new();
    for placement in placements
        .iter()
        .filter(|placement| !placement.archived && placement.unit_of_measure == "pcs")
    {
        let container = containers.get(&placement.container_uuid);
        labels.insert(placement_short_label(placement, container));
    }
    labels.into_iter().collect::<Vec<_>>().join(", ")
}

fn placement_short_label(
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

fn display_part_label(part: &Part) -> String {
    if !part.manufacturer_part_number.is_empty() {
        part.manufacturer_part_number.clone()
    } else if !part.display_value.is_empty() {
        part.display_value.clone()
    } else if !part.internal_part_number.is_empty() {
        part.internal_part_number.clone()
    } else {
        part.entry_uuid.clone()
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

fn is_valid_product_url(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    Url::parse(value)
        .ok()
        .is_some_and(|url| matches!(url.scheme(), "http" | "https"))
}

fn build_order_sheet(
    worksheet: &mut Worksheet,
    rows: &[LabOrderRow],
    formats: &OrderFormats,
) -> Result<(), XlsxError> {
    worksheet.set_name(ORDER_SHEET)?;
    worksheet.set_landscape();
    worksheet.set_print_fit_to_pages(1, 0);
    worksheet.set_freeze_panes(1, 0)?;
    worksheet.set_row_height(0, 30.0)?;

    for (column_index, header) in ORDER_HEADERS.iter().enumerate() {
        let column_index = column_index as u16;
        worksheet.set_column_width(column_index, column_width(header))?;
        worksheet.write_string_with_format(0, column_index, *header, &formats.header)?;
    }

    for (row_index, row) in rows.iter().enumerate() {
        let worksheet_row = row_index as u32 + 1;
        worksheet.set_row_height(worksheet_row, 24.0)?;
        let text_format = formats.text(row_index);
        let number_format = formats.number(row_index);

        write_text(
            worksheet,
            worksheet_row,
            0,
            &row.component_type,
            text_format,
        )?;
        write_text(worksheet, worksheet_row, 1, &row.value, text_format)?;
        write_text(worksheet, worksheet_row, 2, &row.manufacturer, text_format)?;
        write_text(worksheet, worksheet_row, 3, &row.mpn, text_format)?;
        write_text(worksheet, worksheet_row, 4, &row.supplier, text_format)?;
        write_text(worksheet, worksheet_row, 5, &row.supplier_sku, text_format)?;

        if is_valid_product_url(&row.product_link) {
            worksheet.write_url_with_options(
                worksheet_row,
                6,
                row.product_link.as_str(),
                row.product_link.as_str(),
                "",
                Some(text_format),
            )?;
        } else {
            write_text(worksheet, worksheet_row, 6, &row.product_link, text_format)?;
        }

        write_text(worksheet, worksheet_row, 7, &row.stock_status, text_format)?;
        worksheet.write_number_with_format(
            worksheet_row,
            8,
            f64::from(row.current_quantity),
            number_format,
        )?;
        worksheet.write_number_with_format(
            worksheet_row,
            9,
            f64::from(row.requested_quantity),
            number_format,
        )?;
        write_text(worksheet, worksheet_row, 10, &row.location, text_format)?;
        write_text(worksheet, worksheet_row, 11, &row.note, text_format)?;
    }

    worksheet.autofilter(0, 0, rows.len() as u32, (ORDER_HEADERS.len() - 1) as u16)?;
    Ok(())
}

fn write_text(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: &str,
    format: &Format,
) -> Result<(), XlsxError> {
    // Always write as string so formula-like text is never evaluated.
    worksheet.write_string_with_format(row, column, value, format)?;
    Ok(())
}

fn column_width(header: &str) -> f64 {
    match header {
        "Notes" => 34.0,
        "Product Link" => 38.0,
        "MPN" | "Supplier SKU" => 24.0,
        "Location" => 18.0,
        _ => header.chars().count().clamp(12, 22) as f64,
    }
}

fn export_error(error: impl std::fmt::Display) -> String {
    format!("Excel export failed: {error}")
}

struct OrderFormats {
    header: Format,
    number_even: Format,
    number_odd: Format,
    text_even: Format,
    text_odd: Format,
}

impl OrderFormats {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_lab_components::{
        catalog_migration::ensure_catalog_initialized,
        catalog_model::{
            create_part, normalize_part_input, PartInput, StockPlacementInput, StorageAreaInput,
            StorageContainerInput,
        },
        catalog_mutations::{
            create_stock_placement_in_store, create_storage_area_in_store,
            create_storage_container_in_store,
        },
    };
    use std::{
        env, fs,
        fs::File,
        io::Read,
        path::{Path, PathBuf},
    };
    use uuid::Uuid;
    use zip::ZipArchive;

    #[test]
    fn resolves_only_selected_active_parts_from_authoritative_records() {
        let db = catalog_with_order_parts();
        let rows = build_lab_order_rows(
            &db,
            &[request("capacitor-part", 25, "Equivalent acceptable")],
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].component_type, "Capacitor");
        assert_eq!(rows[0].current_quantity, 3);
        assert_eq!(rows[0].requested_quantity, 25);
        assert_eq!(rows[0].location, "A1");
        assert_eq!(rows[0].note, "Equivalent acceptable");
        assert_eq!(rows[0].product_link, "https://supplier.example/part");
        // Part notes must not leak into the transient order note.
        assert_ne!(rows[0].note, "part notes should stay out of order export");
    }

    #[test]
    fn rejects_duplicate_unknown_archived_and_non_piece_requests() {
        let db = catalog_with_order_parts();
        assert!(build_lab_order_rows(&db, &[])
            .unwrap_err()
            .contains("at least one"));
        assert!(build_lab_order_rows(&db, &[request("missing", 1, "")])
            .unwrap_err()
            .contains("not found"));
        assert!(
            build_lab_order_rows(&db, &[request("archived-part", 1, "")])
                .unwrap_err()
                .contains("archived")
        );
        assert!(build_lab_order_rows(&db, &[request("wire-part", 1, "")])
            .unwrap_err()
            .contains("whole pieces"));
        assert!(build_lab_order_rows(
            &db,
            &[
                request("capacitor-part", 1, ""),
                request("capacitor-part", 2, "")
            ]
        )
        .unwrap_err()
        .contains("duplicate"));
        assert!(
            build_lab_order_rows(&db, &[request("capacitor-part", 0, "")])
                .unwrap_err()
                .contains("greater than zero")
        );
    }

    #[test]
    fn writes_focused_order_sheet_with_numeric_quantities_and_hyperlink() {
        let path = temp_xlsx_path("lab-order");
        write_lab_order_workbook(&[order_row()], &path).unwrap();

        assert_eq!(
            workbook_sheet_names(&path),
            vec!["Order Request".to_string()]
        );
        let shared_strings = shared_strings(&path);
        let rows = worksheet_rows(&path, "xl/worksheets/sheet1.xml", &shared_strings);
        assert_eq!(
            rows[0],
            ORDER_HEADERS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(rows[1][0], "Capacitor");
        assert_eq!(worksheet_numeric_cell(&path, 1, 8), 3.0);
        assert_eq!(worksheet_numeric_cell(&path, 1, 9), 25.0);
        assert!(worksheet_has_autofilter(&path));
        assert!(worksheet_has_frozen_header(&path));
        assert!(worksheet_has_hyperlink(
            &path,
            1,
            6,
            "https://supplier.example/part"
        ));

        let worksheet_xml = read_xlsx_member(&path, "xl/worksheets/sheet1.xml");
        assert!(!worksheet_xml.contains("<f>"));
        assert!(!worksheet_xml.contains("<f "));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn building_and_writing_an_order_does_not_change_the_database() {
        let db = catalog_with_order_parts();
        let before = catalog_fingerprint(&db);
        let rows = build_lab_order_rows(&db, &[request("capacitor-part", 25, "")]).unwrap();
        write_lab_order_workbook(&rows, temp_xlsx_path("read-only")).unwrap();
        assert_eq!(catalog_fingerprint(&db), before);
    }

    fn request(part_uuid: &str, requested_quantity: u32, note: &str) -> LabOrderRequestLineInput {
        LabOrderRequestLineInput {
            part_uuid: part_uuid.to_string(),
            requested_quantity,
            note: note.to_string(),
        }
    }

    fn order_row() -> LabOrderRow {
        LabOrderRow {
            component_type: "Capacitor".to_string(),
            value: "100 nF".to_string(),
            manufacturer: "Murata".to_string(),
            mpn: "GRM188R71C104KA01D".to_string(),
            supplier: "DigiKey".to_string(),
            supplier_sku: "490-1551-1-ND".to_string(),
            product_link: "https://supplier.example/part".to_string(),
            stock_status: "in_stock".to_string(),
            current_quantity: 3,
            requested_quantity: 25,
            location: "A1".to_string(),
            note: "Equivalent acceptable".to_string(),
        }
    }

    fn catalog_with_order_parts() -> LabInventoryDb {
        let root = env::temp_dir().join(format!("lab-order-{}", Uuid::new_v4().simple()));
        fs::create_dir_all(&root).unwrap();
        let db = LabInventoryDb::open_at(root.join("te-lab-components.feox")).unwrap();
        ensure_catalog_initialized(&db).unwrap();

        put_part(
            &db,
            "capacitor-part",
            PartInput {
                category: "Passive".to_string(),
                subcategory: "Capacitor".to_string(),
                manufacturer: "Murata".to_string(),
                manufacturer_part_number: "GRM188R71C104KA01D".to_string(),
                display_value: "100 nF".to_string(),
                mounting_type: "smd".to_string(),
                description: "Capacitor 100 nF".to_string(),
                supplier: "DigiKey".to_string(),
                supplier_sku: "490-1551-1-ND".to_string(),
                product_url: "https://supplier.example/part".to_string(),
                default_unit_of_measure: "pcs".to_string(),
                part_status: "active".to_string(),
                notes: "part notes should stay out of order export".to_string(),
                ..PartInput::default()
            },
        );
        put_part(
            &db,
            "archived-part",
            PartInput {
                category: "Passive".to_string(),
                subcategory: "Resistor".to_string(),
                manufacturer_part_number: "ARCHIVED-1".to_string(),
                display_value: "10 kΩ".to_string(),
                default_unit_of_measure: "pcs".to_string(),
                part_status: "obsolete".to_string(),
                archived: true,
                ..PartInput::default()
            },
        );
        put_part(
            &db,
            "wire-part",
            PartInput {
                category: "Wire".to_string(),
                subcategory: "Hookup".to_string(),
                manufacturer_part_number: "WIRE-22AWG".to_string(),
                display_value: "22 AWG".to_string(),
                default_unit_of_measure: "m".to_string(),
                part_status: "active".to_string(),
                ..PartInput::default()
            },
        );

        let area = create_storage_area_in_store(
            StorageAreaInput {
                name: "Main Lab".to_string(),
                area_type: "lab".to_string(),
                ..StorageAreaInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        let grid_container = create_storage_container_in_store(
            StorageContainerInput {
                area_uuid: area.area_uuid.clone(),
                name: "Cabinet 1".to_string(),
                container_type: "cabinet".to_string(),
                grid_enabled: true,
                row_count: Some(8),
                column_count: Some(8),
                ..StorageContainerInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        let freeform_container = create_storage_container_in_store(
            StorageContainerInput {
                area_uuid: area.area_uuid,
                name: "Reel Rack".to_string(),
                container_type: "bin".to_string(),
                grid_enabled: false,
                ..StorageContainerInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        create_stock_placement_in_store(
            StockPlacementInput {
                part_uuid: "capacitor-part".to_string(),
                container_uuid: grid_container.container_uuid,
                row_index: Some(0),
                column_index: Some(0),
                quantity: 3.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            &db,
        )
        .unwrap();
        create_stock_placement_in_store(
            StockPlacementInput {
                part_uuid: "wire-part".to_string(),
                container_uuid: freeform_container.container_uuid,
                freeform_position: "Reel Bin".to_string(),
                quantity: 12.5,
                unit_of_measure: "m".to_string(),
                ..StockPlacementInput::default()
            },
            &db,
        )
        .unwrap();

        db
    }

    fn put_part(db: &LabInventoryDb, entry_uuid: &str, input: PartInput) {
        let id = db.next_entry_id().unwrap();
        let mut part = create_part(id, normalize_part_input(input));
        part.entry_uuid = entry_uuid.to_string();
        db.put_part(&part).unwrap();
        db.set_next_entry_id(id + 1).unwrap();
        db.flush();
    }

    fn catalog_fingerprint(db: &LabInventoryDb) -> String {
        let parts = db.load_parts().unwrap();
        let placements = db.load_stock_placements().unwrap();
        let areas = db.load_storage_areas().unwrap();
        let containers = db.load_storage_containers().unwrap();
        let next_id = db.next_entry_id().unwrap();
        format!("{next_id}|{parts:?}|{placements:?}|{areas:?}|{containers:?}")
    }

    fn temp_xlsx_path(test_name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "lab-order-{test_name}-{}.xlsx",
            Uuid::new_v4().simple()
        ))
    }

    fn workbook_sheet_names(path: &Path) -> Vec<String> {
        let workbook_xml = read_xlsx_member(path, "xl/workbook.xml");
        workbook_xml
            .split("<sheet ")
            .skip(1)
            .filter_map(|chunk| {
                let name = attr_value(chunk, "name")?;
                Some(name.to_string())
            })
            .collect()
    }

    fn worksheet_numeric_cell(path: &Path, row: u32, column: u16) -> f64 {
        let xml = read_xlsx_member(path, "xl/worksheets/sheet1.xml");
        let cell_ref = format!("{}{}", column_name(column), row + 1);
        let marker = format!(r#"r="{cell_ref}""#);
        let cell_start = xml
            .find(&marker)
            .unwrap_or_else(|| panic!("missing cell {cell_ref}"));
        let cell_xml = &xml[cell_start..];
        let value_start = cell_xml.find("<v>").expect("numeric cell missing <v>") + 3;
        let value_end = cell_xml[value_start..]
            .find("</v>")
            .expect("numeric cell missing </v>");
        cell_xml[value_start..value_start + value_end]
            .parse::<f64>()
            .expect("cell value is not numeric")
    }

    fn worksheet_has_autofilter(path: &Path) -> bool {
        read_xlsx_member(path, "xl/worksheets/sheet1.xml").contains("autoFilter")
    }

    fn worksheet_has_frozen_header(path: &Path) -> bool {
        let xml = read_xlsx_member(path, "xl/worksheets/sheet1.xml");
        xml.contains("sheetView")
            && (xml.contains(r#"state="frozen""#) || xml.contains("ySplit=\"1\""))
    }

    fn worksheet_has_hyperlink(path: &Path, row: u32, column: u16, url: &str) -> bool {
        let sheet_xml = read_xlsx_member(path, "xl/worksheets/sheet1.xml");
        let cell_ref = format!("{}{}", column_name(column), row + 1);
        let has_hyperlink_ref = sheet_xml.contains(&format!(r#"ref="{cell_ref}""#))
            && sheet_xml.to_lowercase().contains("hyperlink");
        let relationships = read_xlsx_member(path, "xl/worksheets/_rels/sheet1.xml.rels");
        has_hyperlink_ref && relationships.contains(url)
    }

    fn column_name(column: u16) -> String {
        let mut index = column as u32;
        let mut label = String::new();
        loop {
            let remainder = (index % 26) as u8;
            label.insert(0, (b'A' + remainder) as char);
            if index < 26 {
                break;
            }
            index = index / 26 - 1;
        }
        label
    }

    fn read_xlsx_member(path: &Path, member_name: &str) -> String {
        let file = File::open(path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let mut member = archive.by_name(member_name).unwrap();
        let mut contents = String::new();
        member.read_to_string(&mut contents).unwrap();
        contents
    }

    fn shared_strings(path: &Path) -> Vec<String> {
        let xml = read_xlsx_member(path, "xl/sharedStrings.xml");
        xml.split("<si")
            .skip(1)
            .map(|block| extract_text_nodes(block.split("</si>").next().unwrap_or_default()))
            .collect()
    }

    fn worksheet_rows(
        path: &Path,
        sheet_member_name: &str,
        shared_strings: &[String],
    ) -> Vec<Vec<String>> {
        let xml = read_xlsx_member(path, sheet_member_name);
        xml.split("<row ")
            .skip(1)
            .map(|row_block| {
                let row_body = row_block.split("</row>").next().unwrap_or_default();
                parse_cells(row_body, shared_strings)
            })
            .collect()
    }

    fn parse_cells(row_body: &str, shared_strings: &[String]) -> Vec<String> {
        let mut cells = Vec::new();
        let mut cursor = row_body;

        while let Some(cell_start) = cursor.find("<c ") {
            cursor = &cursor[cell_start..];
            let Some(tag_end) = cursor.find('>') else {
                break;
            };
            let cell_tag = &cursor[..=tag_end];
            let cell_body_start = tag_end + 1;
            let (cell_body, cursor_start) = if cell_tag.ends_with("/>") {
                ("", cell_body_start)
            } else {
                let Some(cell_end) = cursor[cell_body_start..].find("</c>") else {
                    break;
                };
                (
                    &cursor[cell_body_start..cell_body_start + cell_end],
                    cell_body_start + cell_end + "</c>".len(),
                )
            };
            let col = attr_value(cell_tag, "r")
                .map(column_index)
                .unwrap_or(cells.len());
            if cells.len() <= col {
                cells.resize(col + 1, String::new());
            }
            cells[col] = parse_cell_value(cell_tag, cell_body, shared_strings);
            cursor = &cursor[cursor_start..];
        }

        cells
    }

    fn parse_cell_value(cell_tag: &str, cell_body: &str, shared_strings: &[String]) -> String {
        if cell_tag.contains(r#"t="s""#) {
            let index = extract_value(cell_body)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap();
            return shared_strings[index].clone();
        }

        if cell_tag.contains(r#"t="inlineStr""#) {
            return extract_text_nodes(cell_body);
        }

        extract_value(cell_body).unwrap_or_default()
    }

    fn extract_value(cell_body: &str) -> Option<String> {
        let start = cell_body.find("<v>")? + "<v>".len();
        let end = cell_body[start..].find("</v>")?;
        Some(xml_unescape(&cell_body[start..start + end]))
    }

    fn extract_text_nodes(block: &str) -> String {
        let mut value = String::new();
        let mut cursor = block;

        while let Some(text_start) = cursor.find("<t") {
            cursor = &cursor[text_start..];
            let Some(tag_end) = cursor.find('>') else {
                break;
            };
            cursor = &cursor[tag_end + 1..];
            let Some(text_end) = cursor.find("</t>") else {
                break;
            };
            value.push_str(&xml_unescape(&cursor[..text_end]));
            cursor = &cursor[text_end + "</t>".len()..];
        }

        value
    }

    fn attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
        let pattern = format!(r#"{name}=""#);
        let start = tag.find(&pattern)? + pattern.len();
        let end = tag[start..].find('"')?;
        Some(&tag[start..start + end])
    }

    fn column_index(cell_ref: &str) -> usize {
        let mut index = 0usize;
        for byte in cell_ref
            .bytes()
            .take_while(|byte| byte.is_ascii_alphabetic())
        {
            index = index * 26 + usize::from(byte.to_ascii_uppercase() - b'A' + 1);
        }
        index.saturating_sub(1)
    }

    fn xml_unescape(value: &str) -> String {
        value
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
    }
}
