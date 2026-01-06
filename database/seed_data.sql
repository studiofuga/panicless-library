-- Seed data for testing Panicless Library
-- This script creates sample users, books, and reading records for development/testing

-- Note: Password hashes are generated with Argon2
-- Test user password: "test123"
-- Demo user password: "demo123"

-- Clear existing data (in reverse order of dependencies)
DELETE FROM readings;
DELETE FROM books;
DELETE FROM users;

-- Reset sequences
ALTER SEQUENCE users_id_seq RESTART WITH 1;
ALTER SEQUENCE books_id_seq RESTART WITH 1;
ALTER SEQUENCE readings_id_seq RESTART WITH 1;

-- Insert test users
-- Password for testuser: "test123"
-- Password for demouser: "demo123"
-- Note: These are placeholder hashes - the backend will generate proper Argon2 hashes during registration
INSERT INTO users (username, email, password_hash, full_name) VALUES
    ('testuser', 'test@panicless.local', '$argon2id$v=19$m=19456,t=2,p=1$placeholder$hash', 'Test User'),
    ('demouser', 'demo@panicless.local', '$argon2id$v=19$m=19456,t=2,p=1$placeholder$hash', 'Demo User');

-- Insert books for testuser (user_id=1)
INSERT INTO books (user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description) VALUES
    (1, 'The Rust Programming Language', 'Steve Klabnik, Carol Nichols', '2nd', '978-1-7185-0044-0', 2023, 'No Starch Press', 552, 'English', 'The official book on the Rust programming language, written by the Rust development team.'),
    (1, 'Programming Rust', 'Jim Blandy, Jason Orendorff, Leonora F. S. Tindall', '2nd', '978-1-4920-5254-0', 2021, 'O''Reilly Media', 738, 'English', 'Fast, Safe Systems Development'),
    (1, 'Zero to Production in Rust', 'Luca Palmieri', '1st', '978-0-9562-1804-3', 2022, 'Self-published', 394, 'English', 'An introduction to backend development in Rust'),
    (1, 'Designing Data-Intensive Applications', 'Martin Kleppmann', '1st', '978-1-4493-7332-0', 2017, 'O''Reilly Media', 616, 'English', 'The Big Ideas Behind Reliable, Scalable, and Maintainable Systems'),
    (1, 'Clean Code', 'Robert C. Martin', '1st', '978-0-1323-5088-4', 2008, 'Prentice Hall', 464, 'English', 'A Handbook of Agile Software Craftsmanship'),
    (1, '1984', 'George Orwell', 'Reprint', '978-0-4520-8423-4', 1949, 'Secker & Warburg', 328, 'English', 'A dystopian social science fiction novel');

-- Insert books for demouser (user_id=2)
INSERT INTO books (user_id, title, author, edition, isbn, publication_year, publisher, pages, language, description) VALUES
    (2, 'The Pragmatic Programmer', 'David Thomas, Andrew Hunt', '20th Anniversary', '978-0-1352-4464-6', 2019, 'Addison-Wesley', 352, 'English', 'Your Journey to Mastery'),
    (2, 'Eloquent JavaScript', 'Marijn Haverbeke', '3rd', '978-1-5932-7950-6', 2018, 'No Starch Press', 472, 'English', 'A Modern Introduction to Programming'),
    (2, 'The Phoenix Project', 'Gene Kim, Kevin Behr, George Spafford', '1st', '978-0-9882-6250-3', 2013, 'IT Revolution Press', 345, 'English', 'A Novel about IT, DevOps, and Helping Your Business Win');

-- Insert reading records for testuser
INSERT INTO readings (user_id, book_id, start_date, end_date, rating, notes) VALUES
    -- Completed readings
    (1, 1, '2025-01-01', '2025-02-15', 5, 'Excellent introduction to Rust. Very comprehensive and well-written.'),
    (1, 5, '2024-11-01', '2024-11-20', 5, 'Essential reading for any programmer. Changed how I think about code quality.'),
    (1, 6, '2024-12-20', '2024-12-28', 4, 'Classic dystopian novel. Still relevant today.'),

    -- Currently reading (end_date is NULL)
    (1, 2, '2025-02-20', NULL, NULL, 'Deep dive into advanced Rust concepts. Taking it slow.'),

    -- Re-reading a book (second time)
    (1, 1, '2025-12-01', '2025-12-15', 5, 'Re-reading to prepare for a new Rust project. Still learning new things!');

-- Insert reading records for demouser
INSERT INTO readings (user_id, book_id, start_date, end_date, rating, notes) VALUES
    (2, 7, '2025-01-05', '2025-01-25', 5, 'Great advice that stands the test of time.'),
    (2, 8, '2025-01-26', NULL, NULL, 'Enjoying the interactive examples and clear explanations.');

-- Verify the data
SELECT 'Users created:' AS info, COUNT(*) AS count FROM users;
SELECT 'Books created:' AS info, COUNT(*) AS count FROM books;
SELECT 'Readings created:' AS info, COUNT(*) AS count FROM readings;

-- Show summary statistics
SELECT
    u.username,
    COUNT(DISTINCT b.id) AS total_books,
    COUNT(DISTINCT r.id) AS total_readings,
    COUNT(DISTINCT CASE WHEN r.end_date IS NULL THEN r.id END) AS currently_reading,
    COUNT(DISTINCT CASE WHEN r.end_date IS NOT NULL THEN r.id END) AS completed_readings
FROM users u
LEFT JOIN books b ON u.id = b.user_id
LEFT JOIN readings r ON u.id = r.user_id
GROUP BY u.id, u.username
ORDER BY u.id;
