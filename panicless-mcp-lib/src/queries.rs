use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, PgPool};

#[derive(Debug, FromRow)]
pub struct Book {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub author: Option<String>,
    pub edition: Option<String>,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub publisher: Option<String>,
    pub pages: Option<i32>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct BookWithReadings {
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    pub edition: Option<String>,
    pub isbn: Option<String>,
    pub publication_year: Option<i32>,
    pub publisher: Option<String>,
    pub pages: Option<i32>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub reading_count: i64,
    pub readings_summary: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct Reading {
    pub id: i32,
    pub user_id: i32,
    pub book_id: i32,
    pub book_title: String,
    pub book_author: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub rating: Option<i32>,
    pub notes: Option<String>,
}

pub struct ReadingStats {
    pub total_readings: i64,
    pub completed_readings: i64,
    pub current_readings: i64,
    pub total_books_read: i64,
    pub average_rating: Option<f64>,
    pub books_by_year: Vec<(i32, i64)>,
}

pub async fn search_books(
    pool: &PgPool,
    user_id: i32,
    query: Option<&str>,
    author: Option<&str>,
    year: Option<i32>,
) -> Result<Vec<Book>, sqlx::Error> {
    let mut sql = String::from("SELECT id, user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description, cover_image_url, created_at, updated_at FROM books WHERE user_id = $1");
    let mut param_count = 2;

    if query.is_some() {
        sql.push_str(&format!(" AND (title ILIKE ${} OR author ILIKE ${})", param_count, param_count));
        param_count += 1;
    }

    if author.is_some() {
        sql.push_str(&format!(" AND author ILIKE ${}", param_count));
        param_count += 1;
    }

    if year.is_some() {
        sql.push_str(&format!(" AND publication_year = ${}", param_count));
    }

    sql.push_str(" ORDER BY title LIMIT 100");

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

    query_builder.fetch_all(pool).await
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

pub async fn list_readings(
    pool: &PgPool,
    user_id: i32,
    status: Option<&str>,
    year: Option<i32>,
    limit: Option<usize>
) -> Result<Vec<Reading>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT r.id, r.user_id, r.book_id, r.start_date, r.end_date, r.rating, r.notes,
         b.title as book_title, b.author as book_author
         FROM readings r
         JOIN books b ON r.book_id = b.id
         WHERE r.user_id = $1"
    );

    match status {
        Some("current") => sql.push_str(" AND r.end_date IS NULL"),
        Some("completed") => sql.push_str(" AND r.end_date IS NOT NULL"),
        _ => {}
    }

    if year.is_some() {
        sql.push_str(" AND (EXTRACT(YEAR FROM r.start_date) = $2 OR EXTRACT(YEAR FROM r.end_date) = $2)");
    }

    sql.push_str(" ORDER BY r.start_date DESC");

    if let Some(l) = limit {
        sql.push_str(&format!(" LIMIT {}", l));
    }

    let mut query_builder = sqlx::query_as::<_, Reading>(&sql).bind(user_id);

    if let Some(y) = year {
        query_builder = query_builder.bind(y);
    }

    query_builder.fetch_all(pool).await
}

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

    let books_by_year: Vec<(i32, i64)> = sqlx::query_as(
        "SELECT EXTRACT(YEAR FROM end_date)::INTEGER, COUNT(*)::BIGINT
         FROM readings
         WHERE user_id = $1 AND end_date IS NOT NULL
         GROUP BY EXTRACT(YEAR FROM end_date)
         ORDER BY EXTRACT(YEAR FROM end_date) DESC",
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

pub async fn find_similar_books(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
) -> Result<Vec<Book>, sqlx::Error> {
    sqlx::query_as::<_, Book>(
        "SELECT b2.id, b2.user_id, b2.title, b2.author, b2.edition, b2.isbn, b2.publication_year, b2.publisher, b2.pages, b2.language, b2.description, b2.cover_image_url, b2.created_at, b2.updated_at FROM books b1
         JOIN books b2 ON b1.author = b2.author AND b1.id != b2.id
         WHERE b1.id = $1 AND b1.user_id = $2 AND b2.user_id = $2
         ORDER BY b2.title
         LIMIT 20"
    )
    .bind(book_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

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
    let result = sqlx::query_scalar::<_, i32>(
        "INSERT INTO books (user_id, title, author, isbn, publication_year, publisher, pages, language, description, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
         RETURNING id"
    )
    .bind(user_id)
    .bind(title)
    .bind(author)
    .bind(isbn)
    .bind(publication_year)
    .bind(publisher)
    .bind(pages)
    .bind(language)
    .bind(description)
    .fetch_one(pool)
    .await?;
    Ok(result)
}

pub async fn insert_reading(
    pool: &PgPool,
    user_id: i32,
    book_id: i32,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
) -> Result<i32, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i32>(
        "INSERT INTO readings (user_id, book_id, start_date, end_date, created_at, updated_at)
         VALUES ($1, $2, $3, $4, NOW(), NOW())
         RETURNING id"
    )
    .bind(user_id)
    .bind(book_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?;
    Ok(result)
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
         WHERE id = $3 AND user_id = $4"
    )
    .bind(rating)
    .bind(notes)
    .bind(reading_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
