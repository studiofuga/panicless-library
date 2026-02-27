use chrono::NaiveDate;
use sqlx::PgPool;

use crate::models::{
    Book, BookWithReadings, Reading, ReadingStats, ReadingWithBook, SortOrder, YearStats,
    resolve_order_by,
};

// ---------------------------------------------------------------------------
// Books — search / list
// ---------------------------------------------------------------------------

pub async fn search_books(
    pool: &PgPool,
    user_id: i32,
    query: Option<&str>,
    author: Option<&str>,
    year: Option<i32>,
    sort_by: Option<&str>,
    sort_order: Option<&SortOrder>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Book>, sqlx::Error> {
    let mut sql = String::from("SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE user_id = $1");
    let mut param_count = 2;

    if query.is_some() {
        sql.push_str(&format!(
            " AND (title ILIKE ${} OR author ILIKE ${})",
            param_count, param_count
        ));
        param_count += 1;
    }

    if author.is_some() {
        sql.push_str(&format!(" AND author ILIKE ${}", param_count));
        param_count += 1;
    }

    if year.is_some() {
        sql.push_str(&format!(" AND publication_year = ${}", param_count));
        param_count += 1;
    }

    let books_whitelist: &[(&str, &str)] = &[
        ("title", "title"),
        ("author", "author"),
        ("publication_year", "publication_year"),
        ("pages", "pages"),
        ("publisher", "publisher"),
        ("language", "language"),
    ];
    let (sort_col, sort_dir) =
        resolve_order_by(sort_by, sort_order, books_whitelist, "title", "ASC");

    sql.push_str(&format!(
        " ORDER BY {} {} LIMIT ${} OFFSET ${}",
        sort_col,
        sort_dir,
        param_count,
        param_count + 1
    ));

    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);

    let mut query_builder = sqlx::query_as::<_, Book>(&sql).bind(user_id);

    if let Some(q) = query {
        let search_pattern = format!("%{}%", q);
        query_builder = query_builder.bind(search_pattern);
    }

    if let Some(a) = author {
        let author_pattern = format!("%{}%", a);
        query_builder = query_builder.bind(author_pattern);
    }

    if let Some(y) = year {
        query_builder = query_builder.bind(y);
    }

    query_builder = query_builder.bind(limit_val).bind(offset_val);

    query_builder.fetch_all(pool).await
}

pub async fn advanced_search_books(
    pool: &PgPool,
    user_id: i32,
    title: Option<&str>,
    author: Option<&str>,
    isbn: Option<&str>,
    edition: Option<&str>,
    publication_year: Option<i32>,
    language: Option<&str>,
    publisher: Option<&str>,
    description: Option<&str>,
    sort_by: Option<&str>,
    sort_order: Option<&SortOrder>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Book>, sqlx::Error> {
    let mut sql = String::from("SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE user_id = $1");
    let mut param_count = 2;

    if title.is_some() {
        sql.push_str(&format!(" AND title ILIKE ${}", param_count));
        param_count += 1;
    }

    if author.is_some() {
        sql.push_str(&format!(" AND author ILIKE ${}", param_count));
        param_count += 1;
    }

    if isbn.is_some() {
        sql.push_str(&format!(" AND isbn = ${}", param_count));
        param_count += 1;
    }

    if edition.is_some() {
        sql.push_str(&format!(" AND edition ILIKE ${}", param_count));
        param_count += 1;
    }

    if publication_year.is_some() {
        sql.push_str(&format!(" AND publication_year = ${}", param_count));
        param_count += 1;
    }

    if language.is_some() {
        sql.push_str(&format!(" AND language ILIKE ${}", param_count));
        param_count += 1;
    }

    if publisher.is_some() {
        sql.push_str(&format!(" AND publisher ILIKE ${}", param_count));
        param_count += 1;
    }

    if description.is_some() {
        sql.push_str(&format!(" AND description ILIKE ${}", param_count));
        param_count += 1;
    }

    let books_whitelist: &[(&str, &str)] = &[
        ("title", "title"),
        ("author", "author"),
        ("publication_year", "publication_year"),
        ("pages", "pages"),
        ("publisher", "publisher"),
        ("language", "language"),
    ];
    let (sort_col, sort_dir) =
        resolve_order_by(sort_by, sort_order, books_whitelist, "title", "ASC");

    sql.push_str(&format!(
        " ORDER BY {} {} LIMIT ${} OFFSET ${}",
        sort_col,
        sort_dir,
        param_count,
        param_count + 1
    ));

    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);

    let mut query_builder = sqlx::query_as::<_, Book>(&sql).bind(user_id);

    if let Some(t) = title {
        let title_pattern = format!("%{}%", t);
        query_builder = query_builder.bind(title_pattern);
    }

    if let Some(a) = author {
        let author_pattern = format!("%{}%", a);
        query_builder = query_builder.bind(author_pattern);
    }

    if let Some(i) = isbn {
        query_builder = query_builder.bind(i);
    }

    if let Some(e) = edition {
        let edition_pattern = format!("%{}%", e);
        query_builder = query_builder.bind(edition_pattern);
    }

    if let Some(y) = publication_year {
        query_builder = query_builder.bind(y);
    }

    if let Some(l) = language {
        let language_pattern = format!("%{}%", l);
        query_builder = query_builder.bind(language_pattern);
    }

    if let Some(p) = publisher {
        let publisher_pattern = format!("%{}%", p);
        query_builder = query_builder.bind(publisher_pattern);
    }

    if let Some(d) = description {
        let description_pattern = format!("%{}%", d);
        query_builder = query_builder.bind(description_pattern);
    }

    query_builder = query_builder.bind(limit_val).bind(offset_val);

    query_builder.fetch_all(pool).await
}

pub async fn list_unread_books(
    pool: &PgPool,
    user_id: i32,
    search: Option<&str>,
    author: Option<&str>,
    year: Option<i32>,
    sort_by: Option<&str>,
    sort_order: Option<&SortOrder>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Book>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT b.id, b.user_id, b.title, b.author, b.edition, b.isbn, b.publication_year, b.publisher, b.pages, b.language, b.description, b.cover_image_url, b.created_at, b.updated_at FROM books b WHERE b.user_id = $1 AND NOT EXISTS (SELECT 1 FROM readings r WHERE r.book_id = b.id)"
    );
    let mut param_count = 2;

    if search.is_some() {
        sql.push_str(&format!(
            " AND (b.title ILIKE ${} OR b.author ILIKE ${})",
            param_count, param_count
        ));
        param_count += 1;
    }

    if author.is_some() {
        sql.push_str(&format!(" AND b.author ILIKE ${}", param_count));
        param_count += 1;
    }

    if year.is_some() {
        sql.push_str(&format!(" AND b.publication_year = ${}", param_count));
        param_count += 1;
    }

    let unread_whitelist: &[(&str, &str)] = &[
        ("title", "b.title"),
        ("author", "b.author"),
        ("publication_year", "b.publication_year"),
        ("pages", "b.pages"),
        ("publisher", "b.publisher"),
        ("language", "b.language"),
    ];
    let (sort_col, sort_dir) =
        resolve_order_by(sort_by, sort_order, unread_whitelist, "b.title", "ASC");

    sql.push_str(&format!(
        " ORDER BY {} {} LIMIT ${} OFFSET ${}",
        sort_col,
        sort_dir,
        param_count,
        param_count + 1
    ));

    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);

    let mut query_builder = sqlx::query_as::<_, Book>(&sql).bind(user_id);

    if let Some(s) = search {
        let search_pattern = format!("%{}%", s);
        query_builder = query_builder.bind(search_pattern);
    }

    if let Some(a) = author {
        let author_pattern = format!("%{}%", a);
        query_builder = query_builder.bind(author_pattern);
    }

    if let Some(y) = year {
        query_builder = query_builder.bind(y);
    }

    query_builder = query_builder.bind(limit_val).bind(offset_val);

    query_builder.fetch_all(pool).await
}

// ---------------------------------------------------------------------------
// Books — single-record CRUD
// ---------------------------------------------------------------------------

pub async fn get_book(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        "SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE id = $1 AND user_id = $2"
    )
    .bind(book_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_book_with_readings(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
) -> Result<Option<BookWithReadings>, sqlx::Error> {
    sqlx::query_as::<_, BookWithReadings>(
        "SELECT
            b.id,
            b.title,
            b.author,
            b.edition,
            b.isbn,
            b.publication_year,
            b.publisher,
            b.pages,
            b.language,
            b.description,
            COUNT(r.id) as reading_count,
            STRING_AGG(
                CASE
                    WHEN r.end_date IS NULL THEN CONCAT('Currently reading (started ', r.start_date, ')')
                    ELSE CONCAT(r.start_date, ' to ', r.end_date,
                               CASE WHEN r.rating IS NOT NULL THEN CONCAT(' - ', r.rating, '/5') ELSE '' END)
                END,
                E'\n'
            ) as readings_summary
        FROM books b
        LEFT JOIN readings r ON b.id = r.book_id
        WHERE b.id = $1 AND b.user_id = $2
        GROUP BY b.id"
    )
    .bind(book_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_book(
    pool: &PgPool,
    user_id: i32,
    title: &str,
    author: Option<&str>,
    edition: Option<&str>,
    isbn: Option<&str>,
    publication_year: Option<i32>,
    publisher: Option<&str>,
    pages: Option<i32>,
    language: Option<&str>,
    description: Option<&str>,
    cover_image_url: Option<&str>,
) -> Result<Book, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        "INSERT INTO books (user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at"
    )
    .bind(user_id)
    .bind(title)
    .bind(author)
    .bind(edition)
    .bind(isbn)
    .bind(publication_year)
    .bind(publisher)
    .bind(pages)
    .bind(language)
    .bind(description)
    .bind(cover_image_url)
    .fetch_one(pool)
    .await
}

pub async fn update_book(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
    title: Option<&str>,
    author: Option<&str>,
    edition: Option<&str>,
    isbn: Option<&str>,
    publication_year: Option<i32>,
    publisher: Option<&str>,
    pages: Option<i32>,
    language: Option<&str>,
    description: Option<&str>,
    cover_image_url: Option<&str>,
) -> Result<Option<Book>, sqlx::Error> {
    let mut updates = Vec::new();
    let mut param_count = 1;

    if title.is_some() {
        updates.push(format!("title = ${}", param_count));
        param_count += 1;
    }
    if author.is_some() {
        updates.push(format!("author = ${}", param_count));
        param_count += 1;
    }
    if edition.is_some() {
        updates.push(format!("edition = ${}", param_count));
        param_count += 1;
    }
    if isbn.is_some() {
        updates.push(format!("isbn = ${}", param_count));
        param_count += 1;
    }
    if publication_year.is_some() {
        updates.push(format!("publication_year = ${}", param_count));
        param_count += 1;
    }
    if publisher.is_some() {
        updates.push(format!("publisher = ${}", param_count));
        param_count += 1;
    }
    if pages.is_some() {
        updates.push(format!("pages = ${}", param_count));
        param_count += 1;
    }
    if language.is_some() {
        updates.push(format!("language = ${}", param_count));
        param_count += 1;
    }
    if description.is_some() {
        updates.push(format!("description = ${}", param_count));
        param_count += 1;
    }
    if cover_image_url.is_some() {
        updates.push(format!("cover_image_url = ${}", param_count));
        param_count += 1;
    }

    if updates.is_empty() {
        return get_book(pool, user_id, book_id).await;
    }

    updates.push("updated_at = CURRENT_TIMESTAMP".to_string());
    let sql = format!(
        "UPDATE books SET {} WHERE id = ${} AND user_id = ${} RETURNING id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at",
        updates.join(", "),
        param_count,
        param_count + 1
    );

    let mut query_builder = sqlx::query_as::<_, Book>(&sql);

    if let Some(v) = title {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = author {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = edition {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = isbn {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = publication_year {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = publisher {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = pages {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = language {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = description {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = cover_image_url {
        query_builder = query_builder.bind(v);
    }

    query_builder = query_builder.bind(book_id).bind(user_id);

    query_builder.fetch_optional(pool).await
}

pub async fn delete_book(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM books WHERE id = $1 AND user_id = $2")
        .bind(book_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ---------------------------------------------------------------------------
// Books — helpers (kept for MCP tool compatibility)
// ---------------------------------------------------------------------------

pub async fn find_book_by_title(
    pool: &PgPool,
    user_id: i32,
    title: &str,
) -> Result<Vec<Book>, sqlx::Error> {
    let title_pattern = format!("%{}%", title);
    sqlx::query_as::<_, Book>(
        "SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE user_id = $1 AND title ILIKE $2 ORDER BY title"
    )
    .bind(user_id)
    .bind(title_pattern)
    .fetch_all(pool)
    .await
}

pub async fn book_exists(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
) -> Result<bool, sqlx::Error> {
    let result: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM books WHERE id = $1 AND user_id = $2)")
            .bind(book_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(result.0)
}

/// Legacy wrapper kept for MCP tools — returns only the id.
pub async fn insert_book(
    pool: &PgPool,
    user_id: i32,
    title: &str,
    author: Option<&str>,
    isbn: Option<&str>,
    publication_year: Option<i32>,
    publisher: Option<&str>,
    pages: Option<i32>,
    language: Option<&str>,
    description: Option<&str>,
) -> Result<i32, sqlx::Error> {
    let book = create_book(
        pool,
        user_id,
        title,
        author,
        None, // edition
        isbn,
        publication_year,
        publisher,
        pages,
        language,
        description,
        None, // cover_image_url
    )
    .await?;
    Ok(book.id)
}

// ---------------------------------------------------------------------------
// Readings — list
// ---------------------------------------------------------------------------

pub async fn list_readings(
    pool: &PgPool,
    user_id: i32,
    status: Option<&str>,
    year: Option<i32>,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    book_id: Option<i32>,
    sort_by: Option<&str>,
    sort_order: Option<&SortOrder>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ReadingWithBook>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT r.id, r.user_id, r.book_id, r.start_date, r.end_date, r.rating, r.notes, r.created_at, r.updated_at,
         b.title as book_title, b.author as book_author
         FROM readings r
         JOIN books b ON r.book_id = b.id
         WHERE r.user_id = $1"
    );

    let mut param_count = 2;

    match status {
        Some("current") => sql.push_str(" AND r.end_date IS NULL"),
        Some("completed") => sql.push_str(" AND r.end_date IS NOT NULL"),
        _ => {}
    }

    if book_id.is_some() {
        sql.push_str(&format!(" AND r.book_id = ${}", param_count));
        param_count += 1;
    }

    if year.is_some() {
        sql.push_str(&format!(
            " AND (EXTRACT(YEAR FROM r.start_date) = ${} OR EXTRACT(YEAR FROM r.end_date) = ${})",
            param_count, param_count
        ));
        param_count += 1;
    }

    if date_from.is_some() {
        sql.push_str(&format!(" AND r.start_date >= ${}", param_count));
        param_count += 1;
    }

    if date_to.is_some() {
        sql.push_str(&format!(" AND r.end_date <= ${}", param_count));
        param_count += 1;
    }

    let readings_whitelist: &[(&str, &str)] = &[
        ("start_date", "r.start_date"),
        ("end_date", "r.end_date"),
        ("book_title", "b.title"),
        ("book_author", "b.author"),
        ("rating", "r.rating"),
    ];
    let (sort_col, sort_dir) = resolve_order_by(
        sort_by,
        sort_order,
        readings_whitelist,
        "r.start_date",
        "DESC",
    );

    sql.push_str(&format!(
        " ORDER BY {} {} LIMIT ${} OFFSET ${}",
        sort_col,
        sort_dir,
        param_count,
        param_count + 1
    ));

    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);

    let mut query_builder = sqlx::query_as::<_, ReadingWithBook>(&sql).bind(user_id);

    if let Some(bid) = book_id {
        query_builder = query_builder.bind(bid);
    }

    if let Some(y) = year {
        query_builder = query_builder.bind(y);
    }

    if let Some(d) = date_from {
        query_builder = query_builder.bind(d);
    }

    if let Some(d) = date_to {
        query_builder = query_builder.bind(d);
    }

    query_builder = query_builder.bind(limit_val).bind(offset_val);

    query_builder.fetch_all(pool).await
}

// ---------------------------------------------------------------------------
// Readings — single-record CRUD
// ---------------------------------------------------------------------------

pub async fn get_reading(
    pool: &PgPool,
    user_id: i32,
    reading_id: i32,
) -> Result<Option<Reading>, sqlx::Error> {
    sqlx::query_as::<_, Reading>(
        "SELECT id, user_id, book_id, start_date, end_date, rating, notes, created_at, updated_at FROM readings WHERE id = $1 AND user_id = $2"
    )
    .bind(reading_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_book_readings(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
) -> Result<Vec<Reading>, sqlx::Error> {
    sqlx::query_as::<_, Reading>(
        "SELECT id, user_id, book_id, start_date, end_date, rating, notes, created_at, updated_at FROM readings WHERE book_id = $1 AND user_id = $2 ORDER BY start_date DESC"
    )
    .bind(book_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn create_reading(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    rating: Option<i32>,
    notes: Option<&str>,
) -> Result<Reading, sqlx::Error> {
    sqlx::query_as::<_, Reading>(
        "INSERT INTO readings (user_id, book_id, start_date, end_date, rating, notes)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, user_id, book_id, start_date, end_date, rating, notes, created_at, updated_at"
    )
    .bind(user_id)
    .bind(book_id)
    .bind(start_date)
    .bind(end_date)
    .bind(rating)
    .bind(notes)
    .fetch_one(pool)
    .await
}

pub async fn update_reading(
    pool: &PgPool,
    user_id: i32,
    reading_id: i32,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    rating: Option<i32>,
    notes: Option<&str>,
) -> Result<Option<Reading>, sqlx::Error> {
    let mut updates = Vec::new();
    let mut param_count = 1;

    if start_date.is_some() {
        updates.push(format!("start_date = ${}", param_count));
        param_count += 1;
    }
    if end_date.is_some() {
        updates.push(format!("end_date = ${}", param_count));
        param_count += 1;
    }
    if rating.is_some() {
        updates.push(format!("rating = ${}", param_count));
        param_count += 1;
    }
    if notes.is_some() {
        updates.push(format!("notes = ${}", param_count));
        param_count += 1;
    }

    if updates.is_empty() {
        return get_reading(pool, user_id, reading_id).await;
    }

    updates.push("updated_at = CURRENT_TIMESTAMP".to_string());
    let sql = format!(
        "UPDATE readings SET {} WHERE id = ${} AND user_id = ${} RETURNING id, user_id, book_id, start_date, end_date, rating, notes, created_at, updated_at",
        updates.join(", "),
        param_count,
        param_count + 1
    );

    let mut query_builder = sqlx::query_as::<_, Reading>(&sql);

    if let Some(v) = start_date {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = end_date {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = rating {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = notes {
        query_builder = query_builder.bind(v);
    }

    query_builder = query_builder.bind(reading_id).bind(user_id);

    query_builder.fetch_optional(pool).await
}

pub async fn complete_reading(
    pool: &PgPool,
    reading_id: i32,
    end_date: NaiveDate,
    rating: Option<i32>,
) -> Result<Option<Reading>, sqlx::Error> {
    sqlx::query_as::<_, Reading>(
        "UPDATE readings SET end_date = $1, rating = $2, updated_at = CURRENT_TIMESTAMP
         WHERE id = $3
         RETURNING id, user_id, book_id, start_date, end_date, rating, notes, created_at, updated_at"
    )
    .bind(end_date)
    .bind(rating)
    .bind(reading_id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_reading(
    pool: &PgPool,
    user_id: i32,
    reading_id: i32,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM readings WHERE id = $1 AND user_id = $2")
        .bind(reading_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Legacy wrapper kept for MCP tools — returns only the id.
pub async fn insert_reading(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
) -> Result<i32, sqlx::Error> {
    let reading = create_reading(pool, user_id, book_id, start_date, end_date, None, None).await?;
    Ok(reading.id)
}

pub async fn update_reading_review(
    pool: &PgPool,
    user_id: i32,
    reading_id: i32,
    rating: Option<i32>,
    notes: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE readings SET rating = COALESCE($1, rating), notes = COALESCE($2, notes), updated_at = NOW()
         WHERE id = $3 AND user_id = $4",
    )
    .bind(rating)
    .bind(notes)
    .bind(reading_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Readings — stats
// ---------------------------------------------------------------------------

pub async fn get_reading_stats(
    pool: &PgPool,
    user_id: i32,
) -> Result<ReadingStats, sqlx::Error> {
    let total_readings: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM readings WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

    let completed_readings: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM readings WHERE user_id = $1 AND end_date IS NOT NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let current_readings: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM readings WHERE user_id = $1 AND end_date IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let total_books_read: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT book_id) FROM readings WHERE user_id = $1 AND end_date IS NOT NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let avg_rating: (Option<f64>,) = sqlx::query_as(
        "SELECT AVG(rating)::DOUBLE PRECISION FROM readings WHERE user_id = $1 AND rating IS NOT NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let books_by_year: Vec<YearStats> = sqlx::query_as(
        "SELECT EXTRACT(YEAR FROM end_date)::INTEGER as year, COUNT(*)::BIGINT as count
         FROM readings
         WHERE user_id = $1 AND end_date IS NOT NULL
         GROUP BY EXTRACT(YEAR FROM end_date)
         ORDER BY year DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(ReadingStats {
        total_readings: total_readings.0,
        completed_readings: completed_readings.0,
        current_readings: current_readings.0,
        total_books_read: total_books_read.0,
        average_rating: avg_rating.0,
        books_by_year,
    })
}

// ---------------------------------------------------------------------------
// Books — similar
// ---------------------------------------------------------------------------

pub async fn find_similar_books(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Book>, sqlx::Error> {
    let limit_val = limit.unwrap_or(50);
    let offset_val = offset.unwrap_or(0);

    sqlx::query_as::<_, Book>(
        "SELECT b2.id, b2.user_id, b2.title, b2.author, b2.edition, b2.isbn, b2.publication_year, b2.publisher, b2.pages, b2.language, b2.description, b2.cover_image_url, b2.created_at, b2.updated_at FROM books b1
         JOIN books b2 ON b1.author = b2.author AND b1.id != b2.id
         WHERE b1.id = $1 AND b1.user_id = $2 AND b2.user_id = $2
         ORDER BY b2.title
         LIMIT $3 OFFSET $4"
    )
    .bind(book_id)
    .bind(user_id)
    .bind(limit_val)
    .bind(offset_val)
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------
// Bulk operations
// ---------------------------------------------------------------------------

/// Deletes all books and readings for a user. Returns (readings_deleted, books_deleted).
pub async fn delete_all_user_data(
    pool: &PgPool,
    user_id: i32,
) -> Result<(u64, u64), sqlx::Error> {
    let readings_result = sqlx::query("DELETE FROM readings WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    let books_result = sqlx::query("DELETE FROM books WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok((
        readings_result.rows_affected(),
        books_result.rows_affected(),
    ))
}
