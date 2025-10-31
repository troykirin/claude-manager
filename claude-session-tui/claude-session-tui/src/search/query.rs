//! Advanced query parser and search execution with complex filter support

use super::{SearchError, SearchResult, BlockSearchResult, SearchEngine};
use crate::{Role, models::*};
use tantivy::{
    query::{Query, BooleanQuery, TermQuery, RangeQuery, FuzzyTermQuery, RegexQuery, Occur},
    Term, schema::Field, DateTime,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
    str::FromStr,
};
use chrono::{DateTime as ChronoDateTime, Utc};
use regex::Regex;
use tracing::{debug, warn, instrument};

/// Complex search query with filters, sorting, and faceting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Main search text
    pub text: String,
    /// Field-specific filters
    pub filters: Vec<SearchFilter>,
    /// Sort criteria
    pub sort: Vec<SortCriteria>,
    /// Facet requests
    pub facets: Vec<String>,
    /// Result pagination
    pub pagination: Pagination,
    /// Search options
    pub options: SearchOptions,
}

/// Individual search filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
    pub boost: Option<f32>,
}

/// Filter operators for different field types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Regex,
    Range,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    NotIn,
    Exists,
    NotExists,
    Fuzzy,
}

/// Filter values with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Date(ChronoDateTime<Utc>),
    Boolean(bool),
    Array(Vec<String>),
    Range { from: Option<String>, to: Option<String> },
    Regex(String),
}

/// Sort criteria for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortCriteria {
    pub field: String,
    pub direction: SortDirection,
    pub missing_value: Option<SortMissingValue>,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// How to handle missing values in sorting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortMissingValue {
    First,
    Last,
    Value(String),
}

/// Search result pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize,
    pub max_limit: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
            max_limit: 1000,
        }
    }
}

/// Search execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Enable result highlighting
    pub highlight: bool,
    /// Include context information
    pub include_context: bool,
    /// Fuzzy search tolerance
    pub fuzzy_distance: u8,
    /// Query timeout
    pub timeout: Option<Duration>,
    /// Explain query execution
    pub explain: bool,
    /// Minimum relevance score
    pub min_score: Option<f32>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            highlight: true,
            include_context: true,
            fuzzy_distance: 2,
            timeout: Some(Duration::from_secs(10)),
            explain: false,
            min_score: None,
        }
    }
}

/// Search results with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub blocks: Vec<BlockSearchResult>,
    pub facets: HashMap<String, Vec<FacetResult>>,
    pub total_hits: usize,
    pub query_time_ms: u64,
    pub from_cache: bool,
    pub explanation: Option<QueryExplanation>,
    pub aggregations: HashMap<String, AggregationResult>,
}

/// Facet result with counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetResult {
    pub value: String,
    pub count: usize,
    pub selected: bool,
}

/// Query execution explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExplanation {
    pub query_tree: String,
    pub execution_time_breakdown: HashMap<String, u64>,
    pub index_stats: IndexStats,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: usize,
    pub indexed_fields: Vec<String>,
    pub last_update: ChronoDateTime<Utc>,
    pub index_size_mb: f64,
}

/// Aggregation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationResult {
    Terms {
        buckets: Vec<TermsBucket>,
        sum_other_doc_count: usize,
    },
    DateHistogram {
        buckets: Vec<DateHistogramBucket>,
    },
    Stats {
        count: usize,
        min: f64,
        max: f64,
        avg: f64,
        sum: f64,
    },
}

/// Terms aggregation bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsBucket {
    pub key: String,
    pub doc_count: usize,
    pub sub_aggregations: HashMap<String, AggregationResult>,
}

/// Date histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateHistogramBucket {
    pub key: ChronoDateTime<Utc>,
    pub key_as_string: String,
    pub doc_count: usize,
    pub sub_aggregations: HashMap<String, AggregationResult>,
}

/// Query builder for complex searches
pub struct QueryBuilder<'a> {
    engine: &'a SearchEngine,
    text: Option<String>,
    filters: Vec<SearchFilter>,
    sort: Vec<SortCriteria>,
    facets: Vec<String>,
    options: SearchOptions,
    pagination: Pagination,
}

impl SearchQuery {
    /// Create a simple text search query
    pub fn simple(text: &str, limit: Option<usize>) -> Self {
        Self {
            text: text.to_string(),
            filters: Vec::new(),
            sort: vec![SortCriteria {
                field: "_score".to_string(),
                direction: SortDirection::Descending,
                missing_value: None,
            }],
            facets: Vec::new(),
            pagination: Pagination {
                limit: limit.unwrap_or(20),
                ..Default::default()
            },
            options: SearchOptions::default(),
        }
    }

    /// Create a new query builder
    pub fn builder<'a>(engine: &'a SearchEngine) -> QueryBuilder<'a> {
        QueryBuilder::new(engine)
    }

    /// Generate cache key for this query
    pub fn cache_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.text.hash(&mut hasher);
        for filter in &self.filters {
            filter.field.hash(&mut hasher);
            // Hash filter details...
        }
        self.pagination.offset.hash(&mut hasher);
        self.pagination.limit.hash(&mut hasher);
        
        format!("query_{:x}", hasher.finish())
    }

    /// Validate query parameters
    pub fn validate(&self) -> SearchResult<()> {
        if self.text.is_empty() && self.filters.is_empty() {
            return Err(SearchError::query_parsing("Query cannot be empty".to_string(), 0));
        }

        if self.pagination.limit > self.pagination.max_limit {
            return Err(SearchError::query_parsing(
                format!("Limit {} exceeds maximum {}", self.pagination.limit, self.pagination.max_limit),
                0,
            ));
        }

        // Validate filter syntax
        for (i, filter) in self.filters.iter().enumerate() {
            self.validate_filter(filter, i)?;
        }

        // Validate sort fields
        for (i, sort) in self.sort.iter().enumerate() {
            self.validate_sort_field(&sort.field, i)?;
        }

        Ok(())
    }

    fn validate_filter(&self, filter: &SearchFilter, index: usize) -> SearchResult<()> {
        // Validate filter field names
        let valid_fields = [
            "content", "role", "timestamp", "session_id", "block_id",
            "tool_usage", "code_blocks", "file_paths", "topics", "sentiment",
            "complexity_score", "project_path", "language", "intent", "word_count"
        ];

        if !valid_fields.contains(&filter.field.as_str()) {
            return Err(SearchError::query_parsing(
                format!("Invalid filter field: {}", filter.field),
                index,
            ));
        }

        // Validate regex patterns
        if let FilterOperator::Regex = filter.operator {
            if let FilterValue::Regex(pattern) = &filter.value {
                Regex::new(pattern).map_err(|e| {
                    SearchError::query_parsing(
                        format!("Invalid regex pattern: {}", e),
                        index,
                    )
                })?;
            }
        }

        Ok(())
    }

    fn validate_sort_field(&self, field: &str, index: usize) -> SearchResult<()> {
        let sortable_fields = [
            "_score", "timestamp", "complexity_score", "word_count", "relevance"
        ];

        if !sortable_fields.contains(&field) {
            return Err(SearchError::query_parsing(
                format!("Field '{}' is not sortable", field),
                index,
            ));
        }

        Ok(())
    }
}

impl<'a> QueryBuilder<'a> {
    pub fn new(engine: &'a SearchEngine) -> Self {
        Self {
            engine,
            text: None,
            filters: Vec::new(),
            sort: Vec::new(),
            facets: Vec::new(),
            options: SearchOptions::default(),
            pagination: Pagination::default(),
        }
    }

    /// Set the main search text
    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Add a filter to the query
    pub fn filter(mut self, field: &str, operator: FilterOperator, value: FilterValue) -> Self {
        self.filters.push(SearchFilter {
            field: field.to_string(),
            operator,
            value,
            boost: None,
        });
        self
    }

    /// Add a filter with boost
    pub fn filter_with_boost(
        mut self, 
        field: &str, 
        operator: FilterOperator, 
        value: FilterValue,
        boost: f32
    ) -> Self {
        self.filters.push(SearchFilter {
            field: field.to_string(),
            operator,
            value,
            boost: Some(boost),
        });
        self
    }

    /// Filter by role
    pub fn role(self, role: Role) -> Self {
        self.filter("role", FilterOperator::Equals, FilterValue::String(
            match role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::System => "system".to_string(),
                Role::Tool => "tool".to_string(),
            }
        ))
    }

    /// Filter by date range
    pub fn date_range(
        self,
        from: Option<ChronoDateTime<Utc>>,
        to: Option<ChronoDateTime<Utc>>
    ) -> Self {
        self.filter("timestamp", FilterOperator::Range, FilterValue::Range {
            from: from.map(|d| d.to_rfc3339()),
            to: to.map(|d| d.to_rfc3339()),
        })
    }

    /// Filter by project path
    pub fn project_path<S: Into<String>>(self, path: S) -> Self {
        self.filter("project_path", FilterOperator::Contains, FilterValue::String(path.into()))
    }

    /// Filter by tool usage
    pub fn has_tool_usage(self) -> Self {
        self.filter("tool_usage", FilterOperator::Equals, FilterValue::Boolean(true))
    }

    /// Filter by programming language
    pub fn language<S: Into<String>>(self, language: S) -> Self {
        self.filter("language", FilterOperator::Equals, FilterValue::String(language.into()))
    }

    /// Add fuzzy text search
    pub fn fuzzy_text<S: Into<String>>(mut self, text: S, distance: u8) -> Self {
        self.text = Some(text.into());
        self.options.fuzzy_distance = distance;
        self
    }

    /// Add regex filter
    pub fn regex_content<S: Into<String>>(self, pattern: S) -> Self {
        self.filter("content", FilterOperator::Regex, FilterValue::Regex(pattern.into()))
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.pagination.limit = limit;
        self
    }

    /// Set result offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.pagination.offset = offset;
        self
    }

    /// Add sorting criteria
    pub fn sort_by(mut self, field: &str, direction: SortDirection) -> Self {
        self.sort.push(SortCriteria {
            field: field.to_string(),
            direction,
            missing_value: None,
        });
        self
    }

    /// Sort by relevance (default)
    pub fn sort_by_relevance(self) -> Self {
        self.sort_by("_score", SortDirection::Descending)
    }

    /// Sort by timestamp
    pub fn sort_by_date(self, direction: SortDirection) -> Self {
        self.sort_by("timestamp", direction)
    }

    /// Request facets
    pub fn facet<S: Into<String>>(mut self, field: S) -> Self {
        self.facets.push(field.into());
        self
    }

    /// Enable/disable highlighting
    pub fn highlight(mut self, enable: bool) -> Self {
        self.options.highlight = enable;
        self
    }

    /// Set minimum score threshold
    pub fn min_score(mut self, score: f32) -> Self {
        self.options.min_score = Some(score);
        self
    }

    /// Set query timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.options.timeout = Some(timeout);
        self
    }

    /// Enable query explanation
    pub fn explain(mut self, explain: bool) -> Self {
        self.options.explain = explain;
        self
    }

    /// Build and execute the query
    #[instrument(skip(self))]
    pub async fn execute(self) -> SearchResult<SearchResults> {
        let query = SearchQuery {
            text: self.text.unwrap_or_default(),
            filters: self.filters,
            sort: self.sort,
            facets: self.facets,
            pagination: self.pagination,
            options: self.options,
        };

        query.validate()?;
        self.engine.search(&query).await
    }
}

// Predefined query templates for common use cases
impl SearchQuery {
    /// Search for debugging sessions
    pub fn debugging_sessions() -> Self {
        Self::builder_without_engine()
            .filter("intent", FilterOperator::Equals, FilterValue::String("debugging".to_string()))
            .filter("role", FilterOperator::Equals, FilterValue::String("assistant".to_string()))
            .sort_by("timestamp", SortDirection::Descending)
            .build()
    }

    /// Search for code implementations
    pub fn code_implementations(language: Option<&str>) -> Self {
        let mut query = Self::builder_without_engine()
            .filter("code_blocks", FilterOperator::Exists, FilterValue::Boolean(true))
            .filter("intent", FilterOperator::Equals, FilterValue::String("implementation".to_string()));

        if let Some(lang) = language {
            query = query.filter("language", FilterOperator::Equals, FilterValue::String(lang.to_string()));
        }

        query.build()
    }

    /// Search for error patterns
    pub fn error_patterns(error_type: Option<&str>) -> Self {
        let mut query = Self::builder_without_engine()
            .filter("error_patterns", FilterOperator::Exists, FilterValue::Boolean(true));

        if let Some(error_type) = error_type {
            query = query.filter("error_patterns", FilterOperator::Contains, FilterValue::String(error_type.to_string()));
        }

        query.sort_by("timestamp", SortDirection::Descending)
            .build()
    }

    /// Search within a specific project
    pub fn project_search(project_path: &str, text: &str) -> Self {
        Self::builder_without_engine()
            .text(text)
            .filter("project_path", FilterOperator::Contains, FilterValue::String(project_path.to_string()))
            .sort_by_relevance()
            .build()
    }

    // Helper method for templates (would be implemented with proper engine reference in real usage)
    fn builder_without_engine() -> QueryBuilderTemplate {
        QueryBuilderTemplate::new()
    }
}

// Template builder for predefined queries (simplified for templates)
pub struct QueryBuilderTemplate {
    filters: Vec<SearchFilter>,
    sort: Vec<SortCriteria>,
    text: Option<String>,
}

impl QueryBuilderTemplate {
    fn new() -> Self {
        Self {
            filters: Vec::new(),
            sort: Vec::new(),
            text: None,
        }
    }

    fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = Some(text.into());
        self
    }

    fn filter(mut self, field: &str, operator: FilterOperator, value: FilterValue) -> Self {
        self.filters.push(SearchFilter {
            field: field.to_string(),
            operator,
            value,
            boost: None,
        });
        self
    }

    fn sort_by(mut self, field: &str, direction: SortDirection) -> Self {
        self.sort.push(SortCriteria {
            field: field.to_string(),
            direction,
            missing_value: None,
        });
        self
    }

    fn sort_by_relevance(self) -> Self {
        self.sort_by("_score", SortDirection::Descending)
    }

    fn build(self) -> SearchQuery {
        SearchQuery {
            text: self.text.unwrap_or_default(),
            filters: self.filters,
            sort: self.sort,
            facets: Vec::new(),
            pagination: Pagination::default(),
            options: SearchOptions::default(),
        }
    }
}