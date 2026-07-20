#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ModuleId {
    TeTestEquipment,
}

impl ModuleId {
    pub(crate) const TE_SYSTEM_ID_STR: &'static str = "te-test-equipment";

    pub(crate) fn as_system_id_str(self) -> &'static str {
        match self {
            Self::TeTestEquipment => Self::TE_SYSTEM_ID_STR,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            Self::TE_SYSTEM_ID_STR => Some(Self::TeTestEquipment),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ModuleId;

    #[test]
    fn parses_only_the_implemented_te_module() {
        assert_eq!(
            ModuleId::parse("te-test-equipment"),
            Some(ModuleId::TeTestEquipment)
        );
        assert_eq!(
            ModuleId::TeTestEquipment.as_system_id_str(),
            "te-test-equipment"
        );
        assert_eq!(ModuleId::parse("me-storage"), None);
        assert_eq!(ModuleId::parse(""), None);
    }
}
