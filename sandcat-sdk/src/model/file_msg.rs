use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use yew::{html, AttrValue, Html};

use icons::{
    CsvFileIcon, MdFileIcon, PdfFileIcon, TextFileIcon, UnknownFileIcon, XlsFileIcon, ZipFileIcon,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileMsg {
    pub name: String,
    pub server_name: String,
    pub size: usize,
    pub ext: FileExt,
}

impl FileMsg {
    pub fn new(name: String, server_name: String, size: usize, ext: FileExt) -> Self {
        Self {
            name,
            server_name,
            size,
            ext,
        }
    }
}

impl From<&AttrValue> for FileMsg {
    fn from(value: &AttrValue) -> Self {
        // Convert AttrValue to string
        let value_str = value.to_string();

        // Split the string by "||" into parts
        let parts: Vec<&str> = value_str.split("||").collect();

        // Ensure we have exactly 4 parts
        if parts.len() == 4 {
            // Parse size from string to usize
            let size = if let Ok(size) = usize::from_str(parts[2]) {
                size
            } else {
                0
            };
            return FileMsg {
                server_name: parts[0].to_string(),
                name: parts[1].to_string(),
                size,
                ext: FileExt::from_str(parts[3]).unwrap(),
            };
        }

        // If parsing fails, return a default FileMsg
        FileMsg::default()
    }
}

impl Display for FileMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}||{}||{}||{}",
            self.server_name, self.name, self.size, self.ext
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum FileExt {
    #[default]
    Txt,
    Pdf,
    Doc,
    Docx,
    MarkDown,
    MarkDownX,
    Csv,
    Xls,
    Xlsx,
    Zip,
    Unknown(String),
}
impl FromStr for FileExt {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "txt" => Ok(FileExt::Txt),
            "pdf" => Ok(FileExt::Pdf),
            "doc" => Ok(FileExt::Doc),
            "docx" => Ok(FileExt::Docx),
            "md" => Ok(FileExt::MarkDown),
            "mdx" => Ok(FileExt::MarkDownX),
            "csv" => Ok(FileExt::Csv),
            "xls" => Ok(FileExt::Xls),
            "xlsx" => Ok(FileExt::Xlsx),
            "zip" => Ok(FileExt::Zip),
            other => Ok(FileExt::Unknown(other.to_string())),
        }
    }
}

impl Display for FileExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileExt::Txt => write!(f, "txt"),
            FileExt::Pdf => write!(f, "pdf"),
            FileExt::Doc => write!(f, "doc"),
            FileExt::Docx => write!(f, "docx"),
            FileExt::MarkDown => write!(f, "md"),
            FileExt::MarkDownX => write!(f, "mdx"),
            FileExt::Csv => write!(f, "csv"),
            FileExt::Xls => write!(f, "xls"),
            FileExt::Xlsx => write!(f, "xlsx"),
            FileExt::Zip => write!(f, "zip"),
            FileExt::Unknown(ext) => write!(f, "{}", ext),
        }
    }
}

impl FileExt {
    pub fn get_icon(&self) -> Html {
        match self {
            FileExt::Txt => html!(<TextFileIcon/>),
            FileExt::Pdf => html!(<PdfFileIcon/>),
            FileExt::Doc => html!(<TextFileIcon/>),
            FileExt::Docx => html!(<TextFileIcon/>),
            FileExt::MarkDown => html!(<MdFileIcon/>),
            FileExt::MarkDownX => html!(<MdFileIcon/>),
            FileExt::Csv => html!(<CsvFileIcon/>),
            FileExt::Xls => html!(<XlsFileIcon/>),
            FileExt::Xlsx => html!(<XlsFileIcon/>),
            FileExt::Zip => html!(<ZipFileIcon/>),
            FileExt::Unknown(_) => html!(<UnknownFileIcon/>),
        }
    }
}
