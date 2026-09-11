use serde::Deserialize;

#[derive(Deserialize)]
struct CategoryMember {
    title: String,
    ns: i32,
}

#[derive(Deserialize)]
struct CategoryMembersQuery {
    categorymembers: Option<Vec<CategoryMember>>,
}

#[derive(Deserialize)]
struct CategoryMembersResponse {
    query: Option<CategoryMembersQuery>,
}

pub fn fetch_category_members(
    agent: &ureq::Agent,
    category: &str,
    limit: usize,
    timeout_secs: u64,
) -> Result<Vec<String>, super::ApiError> {
    let url = "https://en.wikipedia.org/w/api.php";
    let cmtitle = if category.starts_with("Category:") {
        category.to_string()
    } else {
        format!("Category:{}", category)
    };
    let limit_str = limit.clamp(1, 100).to_string();

    let req = agent
        .get(url)
        .query("action", "query")
        .query("list", "categorymembers")
        .query("cmtitle", &cmtitle)
        .query("cmlimit", &limit_str)
        .query("cmnamespace", "0")
        .query("format", "json");

    let resp: CategoryMembersResponse = super::send_request_json(req, timeout_secs)?;

    let articles = resp
        .query
        .and_then(|q| q.categorymembers)
        .unwrap_or_default()
        .into_iter()
        .filter(|m| m.ns == 0)
        .map(|m| m.title)
        .collect();

    Ok(articles)
}

#[derive(Deserialize)]
struct ArticleCategoryItem {
    title: String,
}

#[derive(Deserialize)]
struct ArticlePageItem {
    title: Option<String>,
    missing: Option<String>,
    categories: Option<Vec<ArticleCategoryItem>>,
}

#[derive(Deserialize)]
struct ArticleCategoriesQuery {
    pages: Option<std::collections::HashMap<String, ArticlePageItem>>,
}

#[derive(Deserialize)]
struct ArticleCategoriesResponse {
    query: Option<ArticleCategoriesQuery>,
}

pub fn fetch_article_categories(
    agent: &ureq::Agent,
    title: &str,
    timeout_secs: u64,
) -> Result<(String, Vec<String>), super::ApiError> {
    let url = "https://en.wikipedia.org/w/api.php";
    let req = agent
        .get(url)
        .query("action", "query")
        .query("prop", "categories")
        .query("clshow", "!hidden")
        .query("titles", title)
        .query("cllimit", "500")
        .query("format", "json");

    let resp: ArticleCategoriesResponse = super::send_request_json(req, timeout_secs)?;

    let page = resp
        .query
        .and_then(|q| q.pages)
        .and_then(|mut p| p.drain().next().map(|(_, v)| v))
        .ok_or_else(|| super::ApiError::NotFound(format!("article {:?} not found", title)))?;

    if page.missing.is_some() {
        return Err(super::ApiError::NotFound(format!("article {:?} not found", title)));
    }

    let display_title = page.title.unwrap_or_else(|| title.to_string());
    let categories = page
        .categories
        .unwrap_or_default()
        .into_iter()
        .map(|c| {
            c.title
                .strip_prefix("Category:")
                .unwrap_or(&c.title)
                .to_string()
        })
        .collect();

    Ok((display_title, categories))
}
