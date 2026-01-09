use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::io::Read;

use crate::models::import::{GoodreadsRecord, TransformedBook};

/// Parse CSV data from bytes
pub fn parse_csv_data<R: Read>(reader: R) -> Result<Vec<Result<GoodreadsRecord, String>>, String> {
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    let mut records = Vec::new();

    for (idx, result) in csv_reader.deserialize().enumerate() {
        match result {
            Ok(record) => records.push(Ok(record)),
            Err(e) => records.push(Err(format!("Row {}: CSV parse error - {}", idx + 2, e))),
        }
    }

    Ok(records)
}

/// Transform a Goodreads record into internal book/reading data
pub fn transform_record(record: &GoodreadsRecord) -> Result<TransformedBook, String> {
    // Title is required
    let title = record.title.trim();
    if title.is_empty() {
        return Err("Title is required".to_string());
    }

    // Use author (prefer over author_lf)
    let author = record
        .author
        .as_ref()
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty());

    // Use ISBN13 if available, otherwise fall back to ISBN
    let isbn = record
        .isbn13
        .as_ref()
        .or(record.isbn.as_ref())
        .map(|i| clean_isbn(i))
        .filter(|i| !i.is_empty());

    // Parse publication year (prefer original publication year)
    let publication_year = record
        .original_publication_year
        .as_ref()
        .or(record.year_published.as_ref())
        .and_then(|y| parse_year(y));

    // Parse pages
    let pages = record
        .number_of_pages
        .as_ref()
        .and_then(|p| parse_integer(p));

    // Publisher
    let publisher = record
        .publisher
        .as_ref()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());

    // Parse rating (0 means no rating, 1-5 are valid ratings)
    let rating = match parse_integer(&record.my_rating) {
        Some(0) => None,
        Some(r) if (1..=5).contains(&r) => Some(r),
        Some(r) => {
            return Err(format!("Invalid rating: {} (must be 0-5)", r));
        }
        None => None,
    };

    // Combine review and private notes
    let notes = combine_notes(&record.my_review, &record.private_notes);

    // Parse dates
    let date_read = record.date_read.as_ref().and_then(|d| parse_date(d));
    let date_added = record.date_added.as_ref().and_then(|d| parse_date(d));

    // Determine start_date and end_date based on shelf
    let shelf = record.exclusive_shelf.trim().to_lowercase();
    let (start_date, end_date) = match shelf.as_str() {
        "read" => {
            // For read books, use date_read as end_date, fallback to date_added
            let end = date_read.or(date_added);
            let start = date_added.or(end);
            (start, end)
        }
        "currently-reading" => {
            // For currently reading, use date_added as start_date, no end_date
            (date_added, None)
        }
        _ => {
            // For to-read or unknown, no reading record will be created
            (None, None)
        }
    };

    Ok(TransformedBook {
        title: title.to_string(),
        author,
        isbn,
        publication_year,
        publisher,
        pages,
        shelf: record.exclusive_shelf.trim().to_string(),
        rating,
        notes,
        start_date,
        end_date,
    })
}

/// Parse a date string in Goodreads format (YYYY/MM/DD)
fn parse_date(date_str: &str) -> Option<NaiveDate> {
    let date_str = date_str.trim();
    if date_str.is_empty() {
        return None;
    }

    // Try YYYY/MM/DD format
    NaiveDate::parse_from_str(date_str, "%Y/%m/%d")
        .ok()
        .or_else(|| {
            // Try YYYY-MM-DD format as fallback
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
        })
}

/// Parse a year string into an integer
fn parse_year(year_str: &str) -> Option<i32> {
    let year_str = year_str.trim();
    if year_str.is_empty() {
        return None;
    }

    // Handle decimal years like "1984.0"
    let year_str = year_str.split('.').next().unwrap_or(year_str);

    year_str.parse::<i32>().ok().filter(|&y| y >= 1000 && y <= 9999)
}

/// Parse an integer string
fn parse_integer(int_str: &str) -> Option<i32> {
    let int_str = int_str.trim();
    if int_str.is_empty() {
        return None;
    }

    int_str.parse::<i32>().ok()
}

/// Clean ISBN by removing hyphens and quotes
fn clean_isbn(isbn: &str) -> String {
    isbn.trim()
        .replace('-', "")
        .replace('\"', "")
        .replace('=', "")
}

/// Combine review and private notes into a single notes field
fn combine_notes(review: &Option<String>, private_notes: &Option<String>) -> Option<String> {
    let review_text = review.as_ref().map(|r| r.trim()).filter(|r| !r.is_empty());
    let notes_text = private_notes
        .as_ref()
        .map(|n| n.trim())
        .filter(|n| !n.is_empty());

    match (review_text, notes_text) {
        (Some(r), Some(n)) => Some(format!("{}\n\n---\n\n{}", r, n)),
        (Some(r), None) => Some(r.to_string()),
        (None, Some(n)) => Some(n.to_string()),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("2024/01/15"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(
            parse_date("2024-01-15"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("invalid"), None);
    }

    #[test]
    fn test_parse_year() {
        assert_eq!(parse_year("2024"), Some(2024));
        assert_eq!(parse_year("2024.0"), Some(2024));
        assert_eq!(parse_year("1984"), Some(1984));
        assert_eq!(parse_year(""), None);
        assert_eq!(parse_year("999"), None); // Too old
        assert_eq!(parse_year("10000"), None); // Too new
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_integer("42"), Some(42));
        assert_eq!(parse_integer("0"), Some(0));
        assert_eq!(parse_integer(""), None);
        assert_eq!(parse_integer("invalid"), None);
    }

    #[test]
    fn test_clean_isbn() {
        assert_eq!(clean_isbn("978-0-547-92822-7"), "9780547928227");
        assert_eq!(clean_isbn("\"=9780547928227\""), "9780547928227");
        assert_eq!(clean_isbn("9780547928227"), "9780547928227");
    }

    #[test]
    fn test_combine_notes() {
        assert_eq!(
            combine_notes(&Some("Review".to_string()), &Some("Notes".to_string())),
            Some("Review\n\n---\n\nNotes".to_string())
        );
        assert_eq!(
            combine_notes(&Some("Review".to_string()), &None),
            Some("Review".to_string())
        );
        assert_eq!(
            combine_notes(&None, &Some("Notes".to_string())),
            Some("Notes".to_string())
        );
        assert_eq!(combine_notes(&None, &None), None);
        assert_eq!(combine_notes(&Some("".to_string()), &Some("".to_string())), None);
    }
}
