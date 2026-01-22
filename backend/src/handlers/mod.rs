pub mod auth;
pub mod books;
pub mod readings;
pub mod users;
pub mod import;
pub mod connectors;
pub mod openapi;
pub mod oauth;
pub mod mcp;

pub use auth::{register, login, refresh, get_current_user};
pub use books::{list_books, advanced_search_books, get_book, create_book, update_book, delete_book, get_book_readings};
pub use readings::{list_readings, get_reading, create_reading, update_reading, delete_reading, complete_reading, get_reading_stats};
pub use users::{get_user, update_user, delete_user};
pub use import::import_goodreads_csv;
pub use connectors::{create_or_update_connector, list_connectors, get_connector, delete_connector, toggle_connector};
pub use openapi::openapi_schema;
pub use oauth::{authorize, token, authorization_server_metadata, protected_resource_metadata};
pub use mcp::{handle_mcp_sse, handle_mcp_sse_post};
