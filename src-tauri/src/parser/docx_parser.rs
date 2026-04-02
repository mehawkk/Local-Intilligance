use crate::config::MAX_TEXT_SIZE;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::Read;
use std::path::Path;

/// Extract text content from a DOCX file.
/// DOCX files are ZIP archives containing word/document.xml.
/// We parse that XML and extract text from <w:t> elements.
/// Uses a single String buffer and stops early when MAX_TEXT_SIZE is reached.
pub fn extract(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open docx: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read docx as zip: {}", e))?;

    let mut xml_content = String::new();
    {
        let mut doc_file = archive
            .by_name("word/document.xml")
            .map_err(|e| format!("No word/document.xml in docx: {}", e))?;
        doc_file
            .read_to_string(&mut xml_content)
            .map_err(|e| format!("Failed to read document.xml: {}", e))?;
    }

    let mut reader = Reader::from_str(&xml_content);
    let mut result = String::with_capacity(4096);
    let mut in_text_element = false;
    let mut buf = Vec::new();

    loop {
        if result.len() >= MAX_TEXT_SIZE {
            break;
        }

        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.local_name();
                let name_bytes = name.as_ref();
                if name_bytes == b"t" {
                    in_text_element = true;
                } else if name_bytes == b"p" && !result.is_empty() {
                    result.push('\n');
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.local_name();
                let name_bytes = name.as_ref();
                if name_bytes == b"t" {
                    in_text_element = false;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text_element {
                    if let Ok(text) = e.unescape() {
                        result.push_str(&text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error in docx: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    if result.len() > MAX_TEXT_SIZE {
        result.truncate(MAX_TEXT_SIZE);
    }

    Ok(result)
}
