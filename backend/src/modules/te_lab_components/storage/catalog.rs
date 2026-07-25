use std::{cmp::Ordering, collections::HashSet};

use serde::de::DeserializeOwned;

use super::{keys, InventoryDb};
use crate::modules::te_lab_components::{
    catalog_model::{
        normalized_lookup, normalized_part_number, Part, StockPlacement, StorageArea,
        StorageContainer,
    },
    model::{db_error, numeric_id, CommandResult},
};

impl InventoryDb {
    pub(crate) fn has_catalog_entities(&self) -> CommandResult<bool> {
        for prefix in [
            keys::PART_PREFIX,
            keys::STORAGE_AREA_PREFIX,
            keys::STORAGE_CONTAINER_PREFIX,
            keys::STOCK_PLACEMENT_PREFIX,
        ] {
            if !self
                .store
                .range_query(prefix.as_bytes(), &keys::range_end_for_prefix(prefix), 1)
                .map_err(db_error)?
                .is_empty()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn load_parts(&self) -> CommandResult<Vec<Part>> {
        let mut parts = self.load_json_prefix(keys::PART_PREFIX)?;
        parts.sort_by(compare_parts);
        Ok(parts)
    }

    pub(crate) fn find_part(&self, part_id: &str) -> CommandResult<Option<Part>> {
        let part_id = part_id.trim();
        if part_id.is_empty() {
            return Ok(None);
        }
        if let Some(part_uuid) = part_id.strip_prefix(keys::PART_PREFIX) {
            return self.get_json(keys::part_key(part_uuid).as_bytes());
        }
        if keys::is_numeric_entry_id(part_id) {
            if let Some(part) = self.find_part_by_index(keys::part_id_key(part_id).as_bytes())? {
                return Ok(Some(part));
            }
        }
        if let Some(part) = self.get_json(keys::part_key(part_id).as_bytes())? {
            return Ok(Some(part));
        }
        if keys::is_numeric_entry_id(part_id) {
            Ok(None)
        } else {
            self.find_part_by_index(keys::part_id_key(part_id).as_bytes())
        }
    }

    pub(crate) fn find_part_by_internal_number(
        &self,
        part_number: &str,
    ) -> CommandResult<Option<Part>> {
        let normalized = normalized_part_number(part_number);
        if normalized.is_empty() {
            return Ok(None);
        }
        self.find_part_by_index(keys::part_number_key(&normalized).as_bytes())
    }

    pub(crate) fn put_part(&self, part: &Part) -> CommandResult<bool> {
        let existing = self.get_json::<Part>(keys::part_key(&part.entry_uuid).as_bytes())?;
        if let Some(existing) = &existing {
            self.delete_part_indexes(existing)?;
        }
        let inserted = self.put_json(keys::part_key(&part.entry_uuid).as_bytes(), part)?;
        if !part.id.is_empty() {
            self.put_bytes(
                keys::part_id_key(&part.id).as_bytes(),
                part.entry_uuid.as_bytes(),
            )?;
        }
        let normalized_number = normalized_part_number(&part.internal_part_number);
        if !normalized_number.is_empty() {
            self.put_bytes(
                keys::part_number_key(&normalized_number).as_bytes(),
                part.entry_uuid.as_bytes(),
            )?;
        }
        Ok(inserted)
    }

    pub(crate) fn delete_part(&self, part: &Part) -> CommandResult<()> {
        self.delete_key(keys::part_key(&part.entry_uuid).as_bytes())?;
        self.delete_part_indexes(part)
    }

    pub(crate) fn load_storage_areas(&self) -> CommandResult<Vec<StorageArea>> {
        let mut areas = self.load_json_prefix::<StorageArea>(keys::STORAGE_AREA_PREFIX)?;
        areas.sort_by(|left, right| {
            left.archived
                .cmp(&right.archived)
                .then_with(|| normalized_lookup(&left.name).cmp(&normalized_lookup(&right.name)))
        });
        Ok(areas)
    }

    pub(crate) fn find_storage_area(&self, area_uuid: &str) -> CommandResult<Option<StorageArea>> {
        self.get_json(keys::storage_area_key(area_uuid.trim()).as_bytes())
    }

    pub(crate) fn find_storage_area_by_name(
        &self,
        name: &str,
    ) -> CommandResult<Option<StorageArea>> {
        let normalized = normalized_lookup(name);
        if normalized.is_empty() {
            return Ok(None);
        }
        self.find_area_by_index(keys::storage_area_name_key(&normalized).as_bytes())
    }

    pub(crate) fn put_storage_area(&self, area: &StorageArea) -> CommandResult<bool> {
        let existing =
            self.get_json::<StorageArea>(keys::storage_area_key(&area.area_uuid).as_bytes())?;
        if let Some(existing) = &existing {
            self.delete_key(
                keys::storage_area_name_key(&normalized_lookup(&existing.name)).as_bytes(),
            )?;
        }
        let inserted = self.put_json(keys::storage_area_key(&area.area_uuid).as_bytes(), area)?;
        if !area.archived {
            self.put_bytes(
                keys::storage_area_name_key(&normalized_lookup(&area.name)).as_bytes(),
                area.area_uuid.as_bytes(),
            )?;
        }
        Ok(inserted)
    }

    pub(crate) fn delete_storage_area(&self, area: &StorageArea) -> CommandResult<()> {
        self.delete_key(keys::storage_area_key(&area.area_uuid).as_bytes())?;
        self.delete_key(keys::storage_area_name_key(&normalized_lookup(&area.name)).as_bytes())
    }

    pub(crate) fn load_storage_containers(&self) -> CommandResult<Vec<StorageContainer>> {
        let mut containers =
            self.load_json_prefix::<StorageContainer>(keys::STORAGE_CONTAINER_PREFIX)?;
        containers.sort_by(|left, right| {
            left.area_uuid
                .cmp(&right.area_uuid)
                .then_with(|| left.archived.cmp(&right.archived))
                .then_with(|| normalized_lookup(&left.name).cmp(&normalized_lookup(&right.name)))
        });
        Ok(containers)
    }

    pub(crate) fn find_storage_container(
        &self,
        container_uuid: &str,
    ) -> CommandResult<Option<StorageContainer>> {
        self.get_json(keys::storage_container_key(container_uuid.trim()).as_bytes())
    }

    pub(crate) fn find_storage_container_by_name(
        &self,
        area_uuid: &str,
        name: &str,
    ) -> CommandResult<Option<StorageContainer>> {
        let normalized = normalized_lookup(name);
        if normalized.is_empty() {
            return Ok(None);
        }
        self.find_container_by_index(
            keys::storage_container_name_key(area_uuid.trim(), &normalized).as_bytes(),
        )
    }

    pub(crate) fn put_storage_container(
        &self,
        container: &StorageContainer,
    ) -> CommandResult<bool> {
        let existing = self.get_json::<StorageContainer>(
            keys::storage_container_key(&container.container_uuid).as_bytes(),
        )?;
        if let Some(existing) = &existing {
            self.delete_key(
                keys::storage_container_name_key(
                    &existing.area_uuid,
                    &normalized_lookup(&existing.name),
                )
                .as_bytes(),
            )?;
        }
        let inserted = self.put_json(
            keys::storage_container_key(&container.container_uuid).as_bytes(),
            container,
        )?;
        if !container.archived {
            self.put_bytes(
                keys::storage_container_name_key(
                    &container.area_uuid,
                    &normalized_lookup(&container.name),
                )
                .as_bytes(),
                container.container_uuid.as_bytes(),
            )?;
        }
        Ok(inserted)
    }

    pub(crate) fn delete_storage_container(
        &self,
        container: &StorageContainer,
    ) -> CommandResult<()> {
        self.delete_key(keys::storage_container_key(&container.container_uuid).as_bytes())?;
        self.delete_key(
            keys::storage_container_name_key(
                &container.area_uuid,
                &normalized_lookup(&container.name),
            )
            .as_bytes(),
        )
    }

    pub(crate) fn load_stock_placements(&self) -> CommandResult<Vec<StockPlacement>> {
        let mut placements = self.load_json_prefix(keys::STOCK_PLACEMENT_PREFIX)?;
        placements.sort_by(compare_placements);
        Ok(placements)
    }

    pub(crate) fn find_stock_placement(
        &self,
        placement_uuid: &str,
    ) -> CommandResult<Option<StockPlacement>> {
        self.get_json(keys::stock_placement_key(placement_uuid.trim()).as_bytes())
    }

    pub(crate) fn load_stock_placements_for_part(
        &self,
        part_uuid: &str,
    ) -> CommandResult<Vec<StockPlacement>> {
        self.load_placements_from_index(&keys::placement_part_index_prefix(part_uuid.trim()))
    }

    pub(crate) fn load_stock_placements_for_container(
        &self,
        container_uuid: &str,
    ) -> CommandResult<Vec<StockPlacement>> {
        self.load_placements_from_index(&keys::placement_container_index_prefix(
            container_uuid.trim(),
        ))
    }

    pub(crate) fn put_stock_placement(&self, placement: &StockPlacement) -> CommandResult<bool> {
        let existing = self.get_json::<StockPlacement>(
            keys::stock_placement_key(&placement.placement_uuid).as_bytes(),
        )?;
        if let Some(existing) = &existing {
            self.delete_placement_indexes(existing)?;
        }
        let inserted = self.put_json(
            keys::stock_placement_key(&placement.placement_uuid).as_bytes(),
            placement,
        )?;
        self.put_bytes(
            keys::placement_part_index_key(&placement.part_uuid, &placement.placement_uuid)
                .as_bytes(),
            placement.placement_uuid.as_bytes(),
        )?;
        self.put_bytes(
            keys::placement_container_index_key(
                &placement.container_uuid,
                &placement.placement_uuid,
            )
            .as_bytes(),
            placement.placement_uuid.as_bytes(),
        )?;
        Ok(inserted)
    }

    pub(crate) fn delete_stock_placement(&self, placement: &StockPlacement) -> CommandResult<()> {
        self.delete_key(keys::stock_placement_key(&placement.placement_uuid).as_bytes())?;
        self.delete_placement_indexes(placement)
    }

    pub(crate) fn replace_catalog_snapshot(
        &self,
        parts: &[Part],
        areas: &[StorageArea],
        containers: &[StorageContainer],
        placements: &[StockPlacement],
    ) -> CommandResult<bool> {
        let changed = self.load_parts()? != parts
            || self.load_storage_areas()? != areas
            || self.load_storage_containers()? != containers
            || self.load_stock_placements()? != placements;
        self.clear_catalog_entities()?;
        let mut seen = HashSet::new();
        for area in areas {
            if seen.insert(format!("area:{}", area.area_uuid)) {
                self.put_storage_area(area)?;
            }
        }
        for container in containers {
            if seen.insert(format!("container:{}", container.container_uuid)) {
                self.put_storage_container(container)?;
            }
        }
        for part in parts {
            if seen.insert(format!("part:{}", part.entry_uuid)) {
                self.put_part(part)?;
            }
        }
        for placement in placements {
            if seen.insert(format!("placement:{}", placement.placement_uuid)) {
                self.put_stock_placement(placement)?;
            }
        }
        let max_id = parts
            .iter()
            .filter_map(|part| part.id.parse::<i64>().ok())
            .max()
            .unwrap_or(0);
        self.set_next_entry_id(max_id + 1)?;
        Ok(changed)
    }

    pub(crate) fn clear_catalog_entities(&self) -> CommandResult<()> {
        for prefix in [
            keys::PART_PREFIX,
            keys::PART_ID_PREFIX,
            keys::PART_NUMBER_PREFIX,
            keys::STORAGE_AREA_PREFIX,
            keys::STORAGE_AREA_NAME_PREFIX,
            keys::STORAGE_CONTAINER_PREFIX,
            keys::STORAGE_CONTAINER_NAME_PREFIX,
            keys::STOCK_PLACEMENT_PREFIX,
            keys::PLACEMENT_PART_INDEX_PREFIX,
            keys::PLACEMENT_CONTAINER_INDEX_PREFIX,
        ] {
            self.clear_prefix(prefix)?;
        }
        Ok(())
    }

    fn find_part_by_index(&self, key: &[u8]) -> CommandResult<Option<Part>> {
        let Some(part_uuid) = self.get_index_uuid(key)? else {
            return Ok(None);
        };
        self.get_json(keys::part_key(&part_uuid).as_bytes())
    }

    fn find_area_by_index(&self, key: &[u8]) -> CommandResult<Option<StorageArea>> {
        let Some(area_uuid) = self.get_index_uuid(key)? else {
            return Ok(None);
        };
        self.find_storage_area(&area_uuid)
    }

    fn find_container_by_index(&self, key: &[u8]) -> CommandResult<Option<StorageContainer>> {
        let Some(container_uuid) = self.get_index_uuid(key)? else {
            return Ok(None);
        };
        self.find_storage_container(&container_uuid)
    }

    fn get_index_uuid(&self, key: &[u8]) -> CommandResult<Option<String>> {
        if !self.store.contains_key(key) {
            return Ok(None);
        }
        String::from_utf8(self.store.get(key).map_err(db_error)?)
            .map(Some)
            .map_err(db_error)
    }

    fn delete_part_indexes(&self, part: &Part) -> CommandResult<()> {
        if !part.id.is_empty() {
            self.delete_key(keys::part_id_key(&part.id).as_bytes())?;
        }
        let normalized_number = normalized_part_number(&part.internal_part_number);
        if !normalized_number.is_empty() {
            self.delete_key(keys::part_number_key(&normalized_number).as_bytes())?;
        }
        Ok(())
    }

    fn delete_placement_indexes(&self, placement: &StockPlacement) -> CommandResult<()> {
        self.delete_key(
            keys::placement_part_index_key(&placement.part_uuid, &placement.placement_uuid)
                .as_bytes(),
        )?;
        self.delete_key(
            keys::placement_container_index_key(
                &placement.container_uuid,
                &placement.placement_uuid,
            )
            .as_bytes(),
        )
    }

    fn load_placements_from_index(&self, prefix: &str) -> CommandResult<Vec<StockPlacement>> {
        let mut placement_uuids = Vec::new();
        self.scan_sync_prefix_from(
            prefix,
            prefix.as_bytes().to_vec(),
            usize::MAX,
            |_, value| {
                placement_uuids.push(String::from_utf8(value.to_vec()).map_err(db_error)?);
                Ok(true)
            },
        )?;
        let mut placements = Vec::new();
        for placement_uuid in placement_uuids {
            if let Some(placement) = self.find_stock_placement(&placement_uuid)? {
                placements.push(placement);
            }
        }
        placements.sort_by(compare_placements);
        Ok(placements)
    }

    fn load_json_prefix<T: DeserializeOwned>(&self, prefix: &str) -> CommandResult<Vec<T>> {
        let mut values = Vec::new();
        self.scan_sync_prefix_from(
            prefix,
            prefix.as_bytes().to_vec(),
            usize::MAX,
            |_, value| {
                values.push(serde_json::from_slice(value).map_err(db_error)?);
                Ok(true)
            },
        )?;
        Ok(values)
    }

    fn clear_prefix(&self, prefix: &str) -> CommandResult<()> {
        let mut keys = Vec::new();
        self.scan_sync_prefix_from(prefix, prefix.as_bytes().to_vec(), usize::MAX, |key, _| {
            keys.push(key.to_vec());
            Ok(true)
        })?;
        for key in keys {
            self.delete_key(&key)?;
        }
        Ok(())
    }
}

fn compare_parts(left: &Part, right: &Part) -> Ordering {
    right
        .updated_at
        .cmp(&left.updated_at)
        .then_with(|| numeric_id(&right.id).cmp(&numeric_id(&left.id)))
}

fn compare_placements(left: &StockPlacement, right: &StockPlacement) -> Ordering {
    left.part_uuid
        .cmp(&right.part_uuid)
        .then_with(|| left.container_uuid.cmp(&right.container_uuid))
        .then_with(|| left.row_index.cmp(&right.row_index))
        .then_with(|| left.column_index.cmp(&right.column_index))
        .then_with(|| left.placement_uuid.cmp(&right.placement_uuid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_lab_components::catalog_model::{
        create_part, create_stock_placement, create_storage_area, create_storage_container,
        normalize_part_input, normalize_stock_placement_input, normalize_storage_area_input,
        normalize_storage_container_input, PartInput, StockPlacementInput, StorageAreaInput,
        StorageContainerInput,
    };
    use std::{env, fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn catalog_entities_round_trip_with_indexes() {
        let db = test_db();
        let part = create_part(
            1,
            normalize_part_input(PartInput {
                internal_part_number: "LAB-001".to_string(),
                description: "1 kOhm resistor".to_string(),
                ..PartInput::default()
            }),
        );
        let area = create_storage_area(normalize_storage_area_input(StorageAreaInput {
            name: "Main Lab".to_string(),
            ..StorageAreaInput::default()
        }));
        let container =
            create_storage_container(normalize_storage_container_input(StorageContainerInput {
                area_uuid: area.area_uuid.clone(),
                name: "Cabinet 1".to_string(),
                grid_enabled: true,
                row_count: Some(4),
                column_count: Some(6),
                ..StorageContainerInput::default()
            }));
        let placement = create_stock_placement(normalize_stock_placement_input(
            StockPlacementInput {
                part_uuid: part.entry_uuid.clone(),
                container_uuid: container.container_uuid.clone(),
                row_index: Some(2),
                column_index: Some(3),
                quantity: 100.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            Some("pcs"),
        ));

        db.put_storage_area(&area).unwrap();
        db.put_storage_container(&container).unwrap();
        db.put_part(&part).unwrap();
        db.put_stock_placement(&placement).unwrap();

        assert_eq!(db.find_part("1").unwrap().unwrap(), part);
        assert_eq!(
            db.find_part_by_internal_number(" lab-001 ")
                .unwrap()
                .unwrap()
                .entry_uuid,
            part.entry_uuid
        );
        assert_eq!(
            db.find_storage_area_by_name("main   lab")
                .unwrap()
                .unwrap()
                .area_uuid,
            area.area_uuid
        );
        assert_eq!(
            db.find_storage_container_by_name(&area.area_uuid, "CABINET 1")
                .unwrap()
                .unwrap()
                .container_uuid,
            container.container_uuid
        );
        assert_eq!(
            db.load_stock_placements_for_part(&part.entry_uuid).unwrap(),
            vec![placement.clone()]
        );
        assert_eq!(
            db.load_stock_placements_for_container(&container.container_uuid)
                .unwrap(),
            vec![placement]
        );
    }

    #[test]
    fn snapshot_replacement_clears_stale_indexes() {
        let db = test_db();
        let first = create_part(
            1,
            normalize_part_input(PartInput {
                internal_part_number: "OLD".to_string(),
                description: "Old".to_string(),
                ..PartInput::default()
            }),
        );
        db.put_part(&first).unwrap();
        assert!(db.find_part_by_internal_number("OLD").unwrap().is_some());

        let second = create_part(
            2,
            normalize_part_input(PartInput {
                internal_part_number: "NEW".to_string(),
                description: "New".to_string(),
                ..PartInput::default()
            }),
        );
        assert!(db
            .replace_catalog_snapshot(std::slice::from_ref(&second), &[], &[], &[])
            .unwrap());

        assert!(db.find_part_by_internal_number("OLD").unwrap().is_none());
        assert_eq!(
            db.find_part_by_internal_number("NEW")
                .unwrap()
                .unwrap()
                .entry_uuid,
            second.entry_uuid
        );
    }

    fn test_db() -> InventoryDb {
        let root = unique_test_dir("catalog-storage");
        fs::create_dir_all(&root).unwrap();
        InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap()
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }
}
