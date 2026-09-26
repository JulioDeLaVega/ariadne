use scraper::{Html, Selector};
use crate::utils::ToolError;

pub fn parse_xhtml(bytes: &[u8]) -> Result<String, ToolError> {
    let html = std::str::from_utf8(bytes)
        .map_err(|_| ToolError::InvalidInput)?;

    let document = Html::parse_document(html);

    let selector = Selector::parse("body")
        .map_err(|_| ToolError::InvalidInput)?;

    let body = document
        .select(&selector)
        .next()
        .ok_or(ToolError::InvalidInput)?;
    
    println!("Body: {:?}", body.inner_html());

    let text = body
        .text()
        .collect::<Vec<_>>()
        .join(" ");

    Ok(text)
}

use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub fn parse_fmx4(bytes: &[u8]) -> Result<String, ToolError> {
    let mut reader = Reader::from_reader(bytes);

    reader.config_mut().trim_text(true);

    let mut buffer = Vec::new();
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Text(e)) => {
                let value = e.as_ref();

                text.push_str(value);
                text.push(' ');
            }

            Ok(Event::Eof) => break,

            Ok(_) => {}

            Err(_) => return Err(ToolError::InvalidInput),
        }

        buffer.clear();
    }

    println!("Input bytes: {}", bytes.len());
    println!("Parsed characters: {}", text.chars().count());

    Ok(text)
}

use lopdf::Document;

pub fn parse_pdf(bytes: &[u8]) -> Result<String, ToolError> {
    let document = Document::load_mem(bytes)
        .map_err(|_| ToolError::InvalidInput)?;

    let pages = document.get_pages();

    let page_numbers: Vec<u32> = pages.keys().copied().collect();

    let text = document
        .extract_text(&page_numbers)
        .map_err(|_| ToolError::InvalidInput)?;

    Ok(text)
}