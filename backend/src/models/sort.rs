use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Validates `sort_by` against a whitelist and returns a safe `(sql_column, sql_direction)` pair.
///
/// - `whitelist` maps parameter names to SQL column expressions (e.g. `("start_date", "r.start_date")`)
/// - Falls back to `default_col` / `default_order` when inputs are `None` or not in whitelist
pub fn resolve_order_by(
    sort_by: Option<&str>,
    sort_order: Option<&SortOrder>,
    whitelist: &[(&str, &str)],
    default_col: &str,
    default_order: &str,
) -> (String, String) {
    let col = sort_by
        .and_then(|key| whitelist.iter().find(|(name, _)| *name == key))
        .map(|(_, sql_expr)| sql_expr.to_string())
        .unwrap_or_else(|| default_col.to_string());

    let dir = match sort_order {
        Some(SortOrder::Asc) => "ASC".to_string(),
        Some(SortOrder::Desc) => "DESC".to_string(),
        None => default_order.to_string(),
    };

    (col, dir)
}
