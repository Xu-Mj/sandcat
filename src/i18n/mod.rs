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
