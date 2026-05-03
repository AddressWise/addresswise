use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt;
use rayon::prelude::*;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::{AutocompleteRequest, AutocompleteResponse, AutocompleteSuggestion};
use crate::normalize::{canonical_country_code, normalize_text};
use crate::repository::AddressRepository;

const DEFAULT_AUTOCOMPLETE_LIMIT: usize = 10;
const MAX_AUTOCOMPLETE_LIMIT: usize = 50;
const DEFAULT_AUTOCOMPLETE_SESSION_TTL: Duration = Duration::from_secs(2 * 60);
const DEFAULT_AUTOCOMPLETE_MAX_SESSION_COUNT: usize = 200_000;
const AUTOCOMPLETE_BUILD_BATCH_SIZE: usize = 100_000;

#[derive(Clone)]
pub(super) struct AutocompleteService {
    index: Arc<AutocompleteIndex>,
    sessions: Arc<RwLock<HashMap<String, AutocompleteSession>>>,
    session_ttl: Duration,
    max_session_count: usize,
}

struct AutocompleteIndex {
    entries: Box<[AutocompleteEntry]>,
    string_pool: StringPool,
    country_order: Box<[u32]>,
    country_ranges: HashMap<CountryCode, Range<usize>>,
    country_codes: Box<[CountryCode]>,
}

#[derive(Debug, Clone)]
struct AutocompleteEntry {
    id: i64,
    street_idx: u32,
    street_norm_idx: u32,
    house_norm_idx: u32, // u32::MAX for None
    locality_idx: u32,   // u32::MAX for None
    postal_idx: u32,     // u32::MAX for None
    formatted_idx: u32,
    country_code: CountryCode,
}

struct StringPool {
    data: Box<[u8]>,
    offsets: Box<[usize]>,
}

impl StringPool {
    fn get(&self, idx: u32) -> &str {
        if idx == u32::MAX {
            return "";
        }
        let start = self.offsets[idx as usize] as usize;
        let end = self.offsets[idx as usize + 1] as usize;
        // Safety: We only put valid UTF-8 into the pool during build
        unsafe { std::str::from_utf8_unchecked(&self.data[start..end]) }
    }
}

struct StringPoolBuilder {
    data: Vec<u8>,
    offsets: Vec<usize>,
    interner: HashMap<Box<str>, u32>,
}

impl StringPoolBuilder {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            offsets: Vec::new(),
            interner: HashMap::new(),
        }
    }

    fn intern(&mut self, s: String) -> u32 {
        if let Some(&idx) = self.interner.get(s.as_str()) {
            idx
        } else {
            let idx = self.offsets.len() as u32;
            self.offsets.push(self.data.len());
            self.data.extend_from_slice(s.as_bytes());
            self.interner.insert(s.into_boxed_str(), idx);
            idx
        }
    }

    fn push(&mut self, s: String) -> u32 {
        let idx = self.offsets.len() as u32;
        self.offsets.push(self.data.len());
        self.data.extend_from_slice(s.as_bytes());
        idx
    }

    fn build(mut self) -> StringPool {
        self.offsets.push(self.data.len());
        StringPool {
            data: self.data.into_boxed_slice(),
            offsets: self.offsets.into_boxed_slice(),
        }
    }
}

struct AutocompleteSession {
    prefix: String,
    scope: SearchScope,
    range: Range<usize>,
    updated_at: Instant,
}

struct PreparedAutocompleteRow {
    id: i64,
    country_code: CountryCode,
    street: String,
    street_norm: String,
    house_norm: String,
    locality: Option<String>,
    postal_code: Option<String>,
    full_address: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct CountryCode([u8; 2]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SearchScope {
    Global,
    Country(CountryCode),
}

impl AutocompleteService {
    pub(super) async fn build(repository: &AddressRepository) -> Result<Self, AppError> {
        let mut entries = Vec::new();
        let mut builder = StringPoolBuilder::new();
        let rayon_threads = rayon::current_num_threads();

        let mut stream = repository.stream_autocomplete();
        let mut batch = Vec::with_capacity(AUTOCOMPLETE_BUILD_BATCH_SIZE);
        let mut processed_rows = 0usize;

        while let Some(row) = stream.next().await {
            batch.push(row?);

            if batch.len() >= AUTOCOMPLETE_BUILD_BATCH_SIZE {
                processed_rows +=
                    Self::process_batch(&mut batch, &mut builder, &mut entries, rayon_threads);
            }
        }

        if !batch.is_empty() {
            processed_rows +=
                Self::process_batch(&mut batch, &mut builder, &mut entries, rayon_threads);
        }

        builder.interner.clear();
        builder.interner.shrink_to_fit();
        let string_pool = builder.build();
        tracing::info!(
            processed_rows,
            entries = entries.len(),
            rayon_threads,
            "finished parallel autocomplete row preparation"
        );

        tracing::info!(
            entries = entries.len(),
            rayon_threads,
            "sorting autocomplete entries in parallel"
        );
        entries.par_sort_unstable_by(|left, right| left.cmp_global(right, &string_pool));

        let mut country_order: Vec<u32> = (0..entries.len() as u32).collect();
        tracing::info!(
            entries = country_order.len(),
            rayon_threads,
            "sorting country-scoped autocomplete order in parallel"
        );
        country_order.par_sort_unstable_by(|left, right| {
            AutocompleteEntry::cmp_country(
                &entries[*left as usize],
                &entries[*right as usize],
                &string_pool,
            )
        });

        let mut country_ranges = HashMap::new();
        let mut country_codes = Vec::new();
        let mut start = 0;
        while start < country_order.len() {
            let country = entries[country_order[start] as usize].country_code;
            let mut end = start + 1;
            while end < country_order.len()
                && entries[country_order[end] as usize].country_code == country
            {
                end += 1;
            }
            country_ranges.insert(country, start..end);
            country_codes.push(country);
            start = end;
        }

        tracing::info!(
            entries = entries.len(),
            pool_bytes = string_pool.data.len(),
            countries = country_ranges.len(),
            "autocomplete index loaded into memory"
        );

        Ok(Self {
            index: Arc::new(AutocompleteIndex {
                entries: entries.into_boxed_slice(),
                string_pool,
                country_order: country_order.into_boxed_slice(),
                country_ranges,
                country_codes: country_codes.into_boxed_slice(),
            }),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_ttl: DEFAULT_AUTOCOMPLETE_SESSION_TTL,
            max_session_count: env_usize(
                "AUTOCOMPLETE_MAX_SESSION_COUNT",
                DEFAULT_AUTOCOMPLETE_MAX_SESSION_COUNT,
            ),
        })
    }

    fn process_batch(
        batch: &mut Vec<crate::repository::AutocompleteAddressRecord>,
        builder: &mut StringPoolBuilder,
        entries: &mut Vec<AutocompleteEntry>,
        rayon_threads: usize,
    ) -> usize {
        let input_rows = batch.len();
        tracing::info!(
            rows = input_rows,
            rayon_threads,
            "preparing autocomplete batch in parallel"
        );

        let prepared_rows = batch
            .drain(..)
            .collect::<Vec<_>>()
            .into_par_iter()
            .filter_map(Self::prepare_row)
            .collect::<Vec<_>>();

        let prepared_count = prepared_rows.len();
        entries.reserve(prepared_count);

        for row in prepared_rows {
            let street_idx = builder.intern(row.street);
            let street_norm_idx = builder.intern(row.street_norm);
            let house_norm_idx = if row.house_norm.is_empty() {
                u32::MAX
            } else {
                builder.intern(row.house_norm)
            };
            let locality_idx = row.locality.map(|s| builder.intern(s)).unwrap_or(u32::MAX);
            let postal_idx = row
                .postal_code
                .map(|s| builder.intern(s))
                .unwrap_or(u32::MAX);
            let formatted_idx = builder.push(row.full_address);

            entries.push(AutocompleteEntry {
                id: row.id,
                street_idx,
                street_norm_idx,
                house_norm_idx,
                locality_idx,
                postal_idx,
                formatted_idx,
                country_code: row.country_code,
            });
        }

        prepared_count
    }

    fn prepare_row(
        row: crate::repository::AutocompleteAddressRecord,
    ) -> Option<PreparedAutocompleteRow> {
        let street = row.thoroughfare?.trim().to_string();
        if street.is_empty() {
            return None;
        }

        let street_norm = normalize_text(&street);
        if street_norm.is_empty() {
            return None;
        }

        let full_norm = normalize_text(&row.full_address);
        let house_norm = if full_norm.starts_with(&street_norm) {
            full_norm[street_norm.len()..].trim().to_string()
        } else {
            String::new()
        };

        let country_code = CountryCode::from_country_code(&row.country_code)?;

        Some(PreparedAutocompleteRow {
            id: row.id,
            country_code,
            street,
            street_norm,
            house_norm,
            locality: row.locality,
            postal_code: row.postal_code,
            full_address: row.full_address,
        })
    }

    pub(super) async fn complete(
        &self,
        request: AutocompleteRequest,
    ) -> Result<AutocompleteResponse, AppError> {
        let session_id = request.session_id.trim();
        if session_id.is_empty() {
            return Err(AppError::bad_request("session_id is required"));
        }

        let query = normalize_text(&request.query);
        if query.is_empty() {
            return Err(AppError::bad_request(
                "query must contain at least one street letter",
            ));
        }

        let scope = match request.country_bias.as_deref().and_then(non_empty) {
            Some(value) => SearchScope::Country(CountryCode::parse(value).ok_or_else(|| {
                AppError::bad_request("country_bias must be a 2-letter ISO country code")
            })?),
            None => SearchScope::Global,
        };

        let limit = request
            .limit
            .unwrap_or(DEFAULT_AUTOCOMPLETE_LIMIT)
            .clamp(1, MAX_AUTOCOMPLETE_LIMIT);

        let mut sessions = self.sessions.write().await;
        self.evict_expired_sessions(&mut sessions);

        let reusable_range = sessions.remove(session_id).and_then(|existing| {
            (existing.scope == scope && query.starts_with(&existing.prefix))
                .then_some(existing.range)
        });

        let range = self.search_range(&query, scope, reusable_range);
        let suggestions = self.collect_suggestions(scope, range.clone(), limit);

        sessions.insert(
            session_id.to_string(),
            AutocompleteSession {
                prefix: query.clone(),
                scope,
                range,
                updated_at: Instant::now(),
            },
        );

        self.enforce_session_limit(&mut sessions);

        Ok(AutocompleteResponse {
            session_id: session_id.to_string(),
            query,
            country_bias: scope.country_code().map(|value| value.to_string()),
            suggestions,
        })
    }

    pub(super) fn country_codes(&self) -> Vec<String> {
        self.index
            .country_codes
            .iter()
            .map(|value| value.to_string())
            .collect()
    }

    fn search_range(
        &self,
        query: &str,
        scope: SearchScope,
        previous: Option<Range<usize>>,
    ) -> Range<usize> {
        let search_range = previous.unwrap_or_else(|| self.full_range(scope));
        let start = self.lower_bound(scope, search_range.clone(), query);
        let end = self.upper_bound(scope, start..search_range.end, query);
        start..end
    }

    fn upper_bound(&self, scope: SearchScope, range: Range<usize>, query: &str) -> usize {
        let mut left = range.start;
        let mut right = range.end;

        while left < right {
            let mid = left + (right - left) / 2;
            let entry = self.entry_at(scope, mid);
            if self.is_match(entry, query) {
                left = mid + 1;
            } else {
                let street = self.index.string_pool.get(entry.street_norm_idx);
                let house = self.index.string_pool.get(entry.house_norm_idx);
                if compare_target(street, house, query) == Ordering::Less {
                    left = mid + 1;
                } else {
                    right = mid;
                }
            }
        }

        left
    }

    fn full_range(&self, scope: SearchScope) -> Range<usize> {
        match scope {
            SearchScope::Global => 0..self.index.entries.len(),
            SearchScope::Country(country) => self
                .index
                .country_ranges
                .get(&country)
                .cloned()
                .unwrap_or(0..0),
        }
    }

    fn lower_bound(&self, scope: SearchScope, range: Range<usize>, query: &str) -> usize {
        let mut left = range.start;
        let mut right = range.end;

        while left < right {
            let mid = left + (right - left) / 2;
            let entry = self.entry_at(scope, mid);
            let street = self.index.string_pool.get(entry.street_norm_idx);
            let house = self.index.string_pool.get(entry.house_norm_idx);

            if compare_target(street, house, query) == Ordering::Less {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        left
    }

    fn is_match(&self, entry: &AutocompleteEntry, query: &str) -> bool {
        let street = self.index.string_pool.get(entry.street_norm_idx);
        let house = self.index.string_pool.get(entry.house_norm_idx);

        if query.len() <= street.len() {
            street.starts_with(query)
        } else if street.eq(&query[..street.len()]) && query.as_bytes()[street.len()] == b' ' {
            house.starts_with(&query[street.len() + 1..])
        } else {
            false
        }
    }

    fn entry_at(&self, scope: SearchScope, position: usize) -> &AutocompleteEntry {
        match scope {
            SearchScope::Global => &self.index.entries[position],
            SearchScope::Country(_) => {
                &self.index.entries[self.index.country_order[position] as usize]
            }
        }
    }

    fn collect_suggestions(
        &self,
        scope: SearchScope,
        range: Range<usize>,
        limit: usize,
    ) -> Vec<AutocompleteSuggestion> {
        let mut suggestions = Vec::with_capacity(limit.min(range.len()));

        for position in range.take(limit) {
            suggestions.push(
                self.entry_at(scope, position)
                    .suggestion(&self.index.string_pool),
            );
        }

        suggestions
    }

    fn evict_expired_sessions(&self, sessions: &mut HashMap<String, AutocompleteSession>) {
        let now = Instant::now();
        sessions.retain(|_, session| now.duration_since(session.updated_at) <= self.session_ttl);
    }

    fn enforce_session_limit(&self, sessions: &mut HashMap<String, AutocompleteSession>) {
        if sessions.len() <= self.max_session_count {
            return;
        }

        let mut ordered = sessions
            .iter()
            .map(|(key, value)| (key.clone(), value.updated_at))
            .collect::<Vec<_>>();
        ordered.sort_by_key(|(_, updated_at)| *updated_at);

        let to_remove = sessions.len() - self.max_session_count;
        for (key, _) in ordered.into_iter().take(to_remove) {
            sessions.remove(&key);
        }
    }
}

fn compare_target(street: &str, house: &str, query: &str) -> Ordering {
    let s_len = street.len();
    let q_len = query.len();
    let cmp_len = s_len.min(q_len);

    match street[..cmp_len].cmp(&query[..cmp_len]) {
        Ordering::Equal => {
            if q_len <= s_len {
                Ordering::Equal
            } else {
                if query.as_bytes()[s_len] == b' ' {
                    if house.is_empty() {
                        Ordering::Less
                    } else {
                        let q_house = &query[s_len + 1..];
                        let h_len = house.len();
                        let qh_len = q_house.len();
                        let h_cmp_len = h_len.min(qh_len);
                        match house[..h_cmp_len].cmp(&q_house[..h_cmp_len]) {
                            Ordering::Equal => {
                                if qh_len <= h_len {
                                    Ordering::Equal
                                } else {
                                    Ordering::Less
                                }
                            }
                            ord => ord,
                        }
                    }
                } else {
                    if b' ' < query.as_bytes()[s_len] {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
            }
        }
        ord => ord,
    }
}

impl AutocompleteEntry {
    fn cmp_global(&self, other: &Self, pool: &StringPool) -> Ordering {
        pool.get(self.street_norm_idx)
            .cmp(pool.get(other.street_norm_idx))
            .then_with(|| {
                pool.get(self.house_norm_idx)
                    .cmp(pool.get(other.house_norm_idx))
            })
            .then_with(|| self.country_code.cmp(&other.country_code))
            .then_with(|| {
                pool.get(self.formatted_idx)
                    .cmp(pool.get(other.formatted_idx))
            })
            .then_with(|| self.id.cmp(&other.id))
    }

    fn cmp_country(left: &Self, right: &Self, pool: &StringPool) -> Ordering {
        left.country_code
            .cmp(&right.country_code)
            .then_with(|| left.cmp_global(right, pool))
    }

    fn suggestion(&self, pool: &StringPool) -> AutocompleteSuggestion {
        AutocompleteSuggestion {
            id: self.id,
            formatted: pool.get(self.formatted_idx).to_string(),
            country_code: self.country_code.to_string(),
            street: pool.get(self.street_idx).to_string(),
            locality: (self.locality_idx != u32::MAX)
                .then(|| pool.get(self.locality_idx).to_string()),
            postal_code: (self.postal_idx != u32::MAX)
                .then(|| pool.get(self.postal_idx).to_string()),
        }
    }
}

impl CountryCode {
    fn parse(value: &str) -> Option<Self> {
        let canonical = canonical_country_code(value)?;
        Self::from_country_code(&canonical)
    }

    fn from_country_code(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        if trimmed.len() != 2 || !trimmed.is_ascii() {
            return None;
        }

        let bytes = trimmed.as_bytes();
        Some(Self([
            bytes[0].to_ascii_uppercase(),
            bytes[1].to_ascii_uppercase(),
        ]))
    }
}

impl SearchScope {
    fn country_code(self) -> Option<CountryCode> {
        match self {
            SearchScope::Global => None,
            SearchScope::Country(country) => Some(country),
        }
    }
}

impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = std::str::from_utf8(&self.0).map_err(|_| std::fmt::Error)?;
        f.write_str(value)
    }
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPool {
        builder: StringPoolBuilder,
    }

    impl TestPool {
        fn new() -> Self {
            Self {
                builder: StringPoolBuilder::new(),
            }
        }

        fn intern(&mut self, s: &str) -> u32 {
            self.builder.intern(s.to_string())
        }

        fn push(&mut self, s: &str) -> u32 {
            self.builder.push(s.to_string())
        }

        fn build(self) -> StringPool {
            self.builder.build()
        }
    }

    fn test_entry(
        pool: &mut TestPool,
        id: i64,
        country_code: &str,
        street: &str,
        formatted: &str,
    ) -> AutocompleteEntry {
        let street_norm = normalize_text(street);
        let full_norm = normalize_text(formatted);
        let house_norm = if full_norm.starts_with(&street_norm) {
            full_norm[street_norm.len()..].trim().to_string()
        } else {
            String::new()
        };

        AutocompleteEntry {
            id,
            country_code: CountryCode::from_country_code(country_code).unwrap(),
            street_idx: pool.intern(street),
            street_norm_idx: pool.intern(&street_norm),
            house_norm_idx: if house_norm.is_empty() {
                u32::MAX
            } else {
                pool.intern(&house_norm)
            },
            formatted_idx: pool.push(formatted),
            locality_idx: u32::MAX,
            postal_idx: u32::MAX,
        }
    }

    fn autocomplete_service(
        mut entries: Vec<AutocompleteEntry>,
        string_pool: StringPool,
    ) -> AutocompleteService {
        entries.sort_by(|a, b| a.cmp_global(b, &string_pool));

        let mut country_order: Vec<u32> = (0..entries.len() as u32).collect();
        country_order.sort_by(|left, right| {
            AutocompleteEntry::cmp_country(
                &entries[*left as usize],
                &entries[*right as usize],
                &string_pool,
            )
        });

        let mut country_ranges = HashMap::new();
        let mut country_codes = Vec::new();
        let mut start = 0;
        while start < country_order.len() {
            let country = entries[country_order[start] as usize].country_code;
            let mut end = start + 1;
            while end < country_order.len()
                && entries[country_order[end] as usize].country_code == country
            {
                end += 1;
            }
            country_ranges.insert(country, start..end);
            country_codes.push(country);
            start = end;
        }

        AutocompleteService {
            index: Arc::new(AutocompleteIndex {
                entries: entries.into_boxed_slice(),
                string_pool,
                country_order: country_order.into_boxed_slice(),
                country_ranges,
                country_codes: country_codes.into_boxed_slice(),
            }),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_ttl: DEFAULT_AUTOCOMPLETE_SESSION_TTL,
            max_session_count: DEFAULT_AUTOCOMPLETE_MAX_SESSION_COUNT,
        }
    }

    #[tokio::test]
    async fn autocomplete_reuses_session_range() {
        let mut pool = TestPool::new();
        let entries = vec![
            test_entry(
                &mut pool,
                1,
                "FR",
                "Avenue de France",
                "Avenue de France 1, Paris, FR",
            ),
            test_entry(
                &mut pool,
                2,
                "FR",
                "Avenue Victor Hugo",
                "Avenue Victor Hugo 2, Paris, FR",
            ),
            test_entry(&mut pool, 3, "US", "Broadway", "Broadway 1, New York, US"),
        ];
        let service = autocomplete_service(entries, pool.build());

        let first = service
            .complete(AutocompleteRequest {
                session_id: "s1".to_string(),
                query: "a".to_string(),
                country_bias: Some("FR".to_string()),
                limit: Some(10),
            })
            .await
            .expect("autocomplete");

        assert_eq!(first.suggestions.len(), 2);

        let second = service
            .complete(AutocompleteRequest {
                session_id: "s1".to_string(),
                query: "avenu".to_string(),
                country_bias: Some("FR".to_string()),
                limit: Some(10),
            })
            .await
            .expect("autocomplete");

        assert_eq!(second.suggestions.len(), 2);

        let third = service
            .complete(AutocompleteRequest {
                session_id: "s1".to_string(),
                query: "avenue d".to_string(),
                country_bias: Some("FR".to_string()),
                limit: Some(10),
            })
            .await
            .expect("autocomplete");

        assert_eq!(third.suggestions.len(), 1);
        assert_eq!(third.suggestions[0].street, "Avenue de France");
    }

    #[tokio::test]
    async fn autocomplete_works_with_house_numbers() {
        let mut pool = TestPool::new();
        let entries = vec![
            test_entry(
                &mut pool,
                1,
                "FR",
                "Avenue de France",
                "Avenue de France 1, Paris, FR",
            ),
            test_entry(
                &mut pool,
                2,
                "FR",
                "Avenue de France",
                "Avenue de France 10, Paris, FR",
            ),
            test_entry(
                &mut pool,
                3,
                "FR",
                "Avenue de France",
                "Avenue de France 2, Paris, FR",
            ),
        ];
        let service = autocomplete_service(entries, pool.build());

        let res = service
            .complete(AutocompleteRequest {
                session_id: "s_hn".to_string(),
                query: "avenue de france 1".to_string(),
                country_bias: Some("FR".to_string()),
                limit: Some(10),
            })
            .await
            .expect("autocomplete");

        // Should find "1" and "10" but not "2"
        assert_eq!(res.suggestions.len(), 2);
        assert!(
            res.suggestions
                .iter()
                .any(|s| s.formatted.contains("France 1,"))
        );
        assert!(
            res.suggestions
                .iter()
                .any(|s| s.formatted.contains("France 10,"))
        );
        assert!(
            !res.suggestions
                .iter()
                .any(|s| s.formatted.contains("France 2,"))
        );
    }

    #[tokio::test]
    async fn autocomplete_restarts_when_country_bias_changes() {
        let mut pool = TestPool::new();
        let entries = vec![
            test_entry(
                &mut pool,
                1,
                "FR",
                "Avenue de France",
                "Avenue de France 1, Paris, FR",
            ),
            test_entry(
                &mut pool,
                2,
                "US",
                "Atlantic Avenue",
                "Atlantic Avenue 1, New York, US",
            ),
        ];
        let service = autocomplete_service(entries, pool.build());

        let first = service
            .complete(AutocompleteRequest {
                session_id: "s2".to_string(),
                query: "a".to_string(),
                country_bias: Some("FR".to_string()),
                limit: Some(10),
            })
            .await
            .expect("autocomplete");

        assert_eq!(first.suggestions.len(), 1);
        assert_eq!(first.suggestions[0].country_code, "FR");

        let second = service
            .complete(AutocompleteRequest {
                session_id: "s2".to_string(),
                query: "at".to_string(),
                country_bias: Some("US".to_string()),
                limit: Some(10),
            })
            .await
            .expect("autocomplete");

        assert_eq!(second.suggestions.len(), 1);
        assert_eq!(second.suggestions[0].country_code, "US");
    }

    #[tokio::test]
    async fn exposes_sorted_unique_country_codes() {
        let mut pool = TestPool::new();
        let entries = vec![
            test_entry(&mut pool, 1, "US", "Broadway", "Broadway 1, New York, US"),
            test_entry(
                &mut pool,
                2,
                "FR",
                "Avenue de France",
                "Avenue de France 1, Paris, FR",
            ),
            test_entry(
                &mut pool,
                3,
                "US",
                "Fifth Avenue",
                "Fifth Avenue 1, New York, US",
            ),
        ];
        let service = autocomplete_service(entries, pool.build());

        assert_eq!(
            service.country_codes(),
            vec!["FR".to_string(), "US".to_string()]
        );
    }
}
