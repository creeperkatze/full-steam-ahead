//! Read-only loader for LiteDB 4 data files (file format version 7).
//!
//! LiteDB only exists for .NET, and the one Rust port targets LiteDB 5. The layout below
//! follows LiteDB 4.1.4's `BasePage`, `DataPage` and `ExtendPage`; all integers are little-endian.

use bson::Document;

const PAGE_SIZE: usize = 4096;
const PAGE_HEADER_SIZE: usize = 25;
const HEADER_INFO: &[u8] = b"** This is a LiteDB file **";
const FILE_VERSION: u8 = 7;
const NO_PAGE: u32 = u32::MAX;
/// Data block header: index (u16), extend page (u32), length (u16).
const BLOCK_HEADER_SIZE: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PageType {
    Header = 1,
    Data = 4,
    Extend = 5,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum LiteDbError {
    #[error("not a LiteDB file")]
    NotLiteDb,
    #[error("unsupported LiteDB file version {0}")]
    UnsupportedVersion(u8),
}

/// Page header: id (u32), type (u8), prev (u32), next (u32), item count (u16), free bytes (u16), 8 reserved.
struct Page<'a> {
    kind: u8,
    next: u32,
    item_count: usize,
    content: &'a [u8],
}

impl Page<'_> {
    fn is(&self, kind: PageType) -> bool {
        self.kind == kind as u8
    }
}

fn page(file: &[u8], id: u32) -> Option<Page<'_>> {
    let start = usize::try_from(id).ok()?.checked_mul(PAGE_SIZE)?;
    let bytes = file.get(start..start.checked_add(PAGE_SIZE)?)?;
    Some(Page {
        kind: bytes[4],
        next: u32::from_le_bytes(bytes[9..13].try_into().ok()?),
        item_count: u16::from_le_bytes(bytes[13..15].try_into().ok()?).into(),
        content: &bytes[PAGE_HEADER_SIZE..],
    })
}

/// Every document stored in the file, across all collections.
///
/// Documents that can't be reassembled or decoded are skipped rather than failing the whole file.
pub fn read_documents(file: &[u8]) -> Result<Vec<Document>, LiteDbError> {
    let header = page(file, 0)
        .filter(|p| p.is(PageType::Header))
        .ok_or(LiteDbError::NotLiteDb)?;
    if header.content.get(..HEADER_INFO.len()) != Some(HEADER_INFO) {
        return Err(LiteDbError::NotLiteDb);
    }
    let version = header.content[HEADER_INFO.len()];
    if version != FILE_VERSION {
        return Err(LiteDbError::UnsupportedVersion(version));
    }

    let page_count = u32::try_from(file.len() / PAGE_SIZE).unwrap_or(u32::MAX);
    let mut documents = Vec::new();
    for id in 1..page_count {
        let Some(data_page) = page(file, id).filter(|p| p.is(PageType::Data)) else {
            continue;
        };
        let mut blocks = data_page.content;
        for _ in 0..data_page.item_count {
            let Some((extend_page, data, rest)) = split_block(blocks) else {
                break;
            };
            blocks = rest;
            let bytes = if extend_page == NO_PAGE {
                Some(data.to_vec())
            } else {
                // Large documents live entirely in extend pages; the block's own data is empty
                read_extend_chain(file, extend_page, page_count)
            };
            if let Some(doc) = bytes.and_then(|b| Document::from_reader(b.as_slice()).ok()) {
                documents.push(doc);
            }
        }
    }
    Ok(documents)
}

/// Splits one data block off the front of a data page's content.
fn split_block(blocks: &[u8]) -> Option<(u32, &[u8], &[u8])> {
    let header = blocks.get(..BLOCK_HEADER_SIZE)?;
    let extend_page = u32::from_le_bytes(header[2..6].try_into().ok()?);
    let len = usize::from(u16::from_le_bytes(header[6..8].try_into().ok()?));
    let data = blocks.get(BLOCK_HEADER_SIZE..BLOCK_HEADER_SIZE + len)?;
    Some((extend_page, data, &blocks[BLOCK_HEADER_SIZE + len..]))
}

/// Concatenates a linked list of extend pages, whose item count is their byte length.
fn read_extend_chain(file: &[u8], first: u32, page_count: u32) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut id = first;
    // A chain can't be longer than the file, so this also stops corrupt loops
    for _ in 0..page_count {
        if id == NO_PAGE {
            return Some(bytes);
        }
        let extend = page(file, id).filter(|p| p.is(PageType::Extend))?;
        bytes.extend_from_slice(extend.content.get(..extend.item_count)?);
        id = extend.next;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::{spec::BinarySubtype, Binary, Bson, DateTime};

    /// Written by LiteDB 4.1.4 itself; see `fixtures/litedb_v4.cs` for how.
    const FIXTURE: &[u8] = include_bytes!("fixtures/litedb_v4.db");

    fn fixture_doc(docs: &[Document], name: &str) -> Document {
        docs.iter()
            .find(|d| d.get_str("Name").ok() == Some(name))
            .unwrap_or_else(|| panic!("{name} missing"))
            .clone()
    }

    /// `new Guid(n, 0x1122, 0x3344, 0x55, ...)` as written by .NET's `Guid.ToByteArray`.
    fn fixture_guid(n: u8) -> Bson {
        Bson::Binary(Binary {
            subtype: BinarySubtype::Uuid,
            bytes: vec![
                n, 0, 0, 0, 0x22, 0x11, 0x44, 0x33, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
            ],
        })
    }

    #[test]
    fn fixture_contains_exactly_the_live_documents() {
        let docs = read_documents(FIXTURE).unwrap();
        let mut names: Vec<_> = docs.iter().map(|d| d.get_str("Name").unwrap()).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "Grown",
                "Large",
                "Not installed",
                "Raw",
                "Typed",
                "Ünïcödé 名前"
            ]
        );
    }

    #[test]
    fn fixture_decodes_every_value_type() {
        let doc = fixture_doc(&read_documents(FIXTURE).unwrap(), "Typed");
        assert_eq!(doc.get("_id"), Some(&fixture_guid(1)));
        assert!(doc.get_bool("IsInstalled").unwrap());
        assert_eq!(doc.get_i32("PlayCount").unwrap(), 42);
        assert_eq!(doc.get_i64("Playtime").unwrap(), 1 << 40);
        assert_eq!(doc.get_f64("Score").unwrap(), 9.5);
        assert!(matches!(doc.get("Price"), Some(Bson::Decimal128(_))));
        assert_eq!(
            doc.get_datetime("Added").unwrap(),
            &DateTime::builder()
                .year(2020)
                .month(1)
                .day(2)
                .hour(3)
                .minute(4)
                .second(5)
                .build()
                .unwrap()
        );
        assert_eq!(
            doc.get_array("Tags").unwrap(),
            &vec![Bson::from("a"), Bson::from("b")]
        );
        assert_eq!(
            doc.get_document("Stats").unwrap().get_i32("wins").unwrap(),
            3
        );
        assert_eq!(doc.get_binary_generic("Blob").unwrap(), &vec![1, 2, 3]);
        assert_eq!(
            doc.get_object_id("Ref").unwrap().to_hex(),
            "5f0000000000000000000001"
        );
    }

    #[test]
    fn fixture_reassembles_documents_from_extend_pages() {
        let docs = read_documents(FIXTURE).unwrap();
        let large = fixture_doc(&docs, "Large");
        assert_eq!(large.get("_id"), Some(&fixture_guid(3)));
        assert_eq!(large.get_str("Description").unwrap(), "x".repeat(10000));
        let grown = fixture_doc(&docs, "Grown");
        assert_eq!(grown.get_str("Description").unwrap(), "y".repeat(5000));
    }

    #[test]
    fn fixture_keeps_unicode_nulls_and_markers() {
        let docs = read_documents(FIXTURE).unwrap();
        assert_eq!(
            fixture_doc(&docs, "Ünïcödé 名前").get("_id"),
            Some(&fixture_guid(6))
        );
        let plain = fixture_doc(&docs, "Not installed");
        assert!(!plain.get_bool("IsInstalled").unwrap());
        // The .NET mapper leaves out null properties
        assert_eq!(plain.get("Added"), None);

        let raw = fixture_doc(&docs, "Raw");
        assert_eq!(raw.get_i32("_id").unwrap(), 7);
        assert_eq!(raw.get("Nothing"), Some(&Bson::Null));
        assert_eq!(raw.get("Min"), Some(&Bson::MinKey));
        assert_eq!(raw.get("Max"), Some(&Bson::MaxKey));
    }

    // Hand-built files for cases a healthy LiteDB file never contains

    fn doc_bytes(name: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        bson::doc! { "Name": name }.to_writer(&mut bytes).unwrap();
        bytes
    }

    fn page_bytes(kind: u8, next: u32, item_count: usize, content: &[u8]) -> Vec<u8> {
        let mut page = vec![0u8; PAGE_SIZE];
        page[4] = kind;
        page[9..13].copy_from_slice(&next.to_le_bytes());
        page[13..15].copy_from_slice(&u16::try_from(item_count).unwrap().to_le_bytes());
        page[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + content.len()].copy_from_slice(content);
        page
    }

    fn header_page(version: u8) -> Vec<u8> {
        let content = [HEADER_INFO, &[version]].concat();
        page_bytes(PageType::Header as u8, NO_PAGE, 0, &content)
    }

    fn block(extend_page: u32, data: &[u8]) -> Vec<u8> {
        let mut block = vec![0, 0];
        block.extend(extend_page.to_le_bytes());
        block.extend(u16::try_from(data.len()).unwrap().to_le_bytes());
        block.extend(data);
        block
    }

    fn names(file: &[u8]) -> Vec<String> {
        read_documents(file)
            .unwrap()
            .iter()
            .map(|d| d.get_str("Name").unwrap().to_string())
            .collect()
    }

    #[test]
    fn rejects_files_that_are_not_litedb() {
        assert_eq!(read_documents(b""), Err(LiteDbError::NotLiteDb));
        assert_eq!(read_documents(&[0; PAGE_SIZE]), Err(LiteDbError::NotLiteDb));
        let mut wrong_info = header_page(FILE_VERSION);
        wrong_info[PAGE_HEADER_SIZE] = b'#';
        assert_eq!(read_documents(&wrong_info), Err(LiteDbError::NotLiteDb));
    }

    #[test]
    fn rejects_other_format_versions() {
        // LiteDB 5 writes version 8 (and uses larger pages)
        assert_eq!(
            read_documents(&header_page(8)),
            Err(LiteDbError::UnsupportedVersion(8))
        );
    }

    #[test]
    fn empty_database_has_no_documents() {
        assert_eq!(read_documents(&header_page(FILE_VERSION)), Ok(Vec::new()));
    }

    #[test]
    fn looping_extend_chain_skips_only_that_document() {
        let data = [block(NO_PAGE, &doc_bytes("Inline")), block(2, &[])].concat();
        let file = [
            header_page(FILE_VERSION),
            page_bytes(PageType::Data as u8, NO_PAGE, 2, &data),
            page_bytes(PageType::Extend as u8, 2, 4, &[1, 2, 3, 4]),
        ]
        .concat();
        assert_eq!(names(&file), ["Inline"]);
    }

    #[test]
    fn extend_chain_through_a_non_extend_page_is_skipped() {
        let data = [block(2, &[]), block(NO_PAGE, &doc_bytes("Inline"))].concat();
        let file = [
            header_page(FILE_VERSION),
            page_bytes(PageType::Data as u8, NO_PAGE, 2, &data),
            page_bytes(PageType::Data as u8, NO_PAGE, 0, &[]),
        ]
        .concat();
        assert_eq!(names(&file), ["Inline"]);
    }

    #[test]
    fn truncated_block_and_trailing_partial_page_are_ignored() {
        let mut data = block(NO_PAGE, &doc_bytes("Inline"));
        // Claims a second block, but its length runs past the page
        data.extend(block(NO_PAGE, &[]));
        let len_at = data.len() - 2;
        data[len_at..].copy_from_slice(&u16::MAX.to_le_bytes());
        let mut file = [
            header_page(FILE_VERSION),
            page_bytes(PageType::Data as u8, NO_PAGE, 2, &data),
        ]
        .concat();
        file.extend([PageType::Data as u8; 100]);
        assert_eq!(names(&file), ["Inline"]);
    }

    #[test]
    fn undecodable_document_is_skipped() {
        let data = [
            block(NO_PAGE, &[5, 0, 0, 0]),
            block(NO_PAGE, &doc_bytes("Ok")),
        ]
        .concat();
        let file = [
            header_page(FILE_VERSION),
            page_bytes(PageType::Data as u8, NO_PAGE, 2, &data),
        ]
        .concat();
        assert_eq!(names(&file), ["Ok"]);
    }
}
