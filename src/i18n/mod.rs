use std::fmt::Display;

/// because of we don't have a lot of resource which need to i18n,
/// so we just use a simple way to i18n
pub mod en_us;
pub mod zh_cn;

#[allow(dead_code)]
/// i18n language type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LanguageType {
    ZhCN,
    #[default]
    EnUS,
}

impl Display for LanguageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageType::ZhCN => write!(f, "zh_cn"),
            LanguageType::EnUS => write!(f, "en_us"),
        }
    }
}

impl From<String> for LanguageType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "zh_cn" => LanguageType::ZhCN,
            "en_us" => LanguageType::EnUS,
            _ => LanguageType::EnUS,
        }
    }
}
