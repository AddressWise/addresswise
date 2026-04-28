use crate::error::AppError;
use crate::index::AddressIndex;
use crate::models::{ResolveAddressRequest, ResolveAddressResponse, StructuredAddressInput};
use crate::normalize::{canonical_country_code, compact_alphanumeric, normalize_text};
use crate::repository::{AddressRepository, AddressSearch};

#[derive(Clone)]
pub struct AddressService {
    repository: AddressRepository,
    index: AddressIndex,
    candidate_limit: i64,
}

#[derive(Debug, Default)]
struct ResolvedInput {
    query: Option<String>,
    house_number: Option<String>,
    street: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

impl AddressService {
    pub fn new(repository: AddressRepository, index: AddressIndex, candidate_limit: i64) -> Self {
        Self {
            repository,
            index,
            candidate_limit,
        }
    }

    pub async fn resolve(
        &self,
        request: ResolveAddressRequest,
    ) -> Result<ResolveAddressResponse, AppError> {
        let input = ResolvedInput::from_request(request);
        let search_params = self.prepare_search(input)?;

        // 1. Search Tantivy for top candidates
        let candidates = self.index.search(&search_params.query, 50)?;
        
        if candidates.is_empty() {
            return Err(AppError::NotFound);
        }

        // 2. Fetch full details from Postgres for these candidates
        let ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();
        let mut results = self.repository.get_by_ids(&ids, &search_params).await?;

        // 3. Pick the best one
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter().next().ok_or(AppError::NotFound)
    }

    fn prepare_search(&self, input: ResolvedInput) -> Result<AddressSearch, AppError> {
        Self::prepare_search_with_limit(input, self.candidate_limit)
    }

    fn prepare_search_with_limit(
        input: ResolvedInput,
        candidate_limit: i64,
    ) -> Result<AddressSearch, AppError> {
        let explicit_country_code = match input.country.as_deref().and_then(non_empty) {
            Some(country) => Some(canonical_country_code(country).ok_or_else(|| {
                AppError::bad_request("country must be a 2-letter ISO country code")
            })?),
            None => None,
        };

        let query = match input.query.as_deref().and_then(non_empty) {
            Some(query) => query.to_string(),
            None => input.structured_query_text(explicit_country_code.as_deref()),
        };
        let query = normalize_text(&query);
        let country_code = explicit_country_code.as_ref().cloned().or_else(|| infer_country_code_from_query(&query));

        if query.len() < 3 || query.split_whitespace().all(|part| part.len() < 2) {
            return Err(AppError::bad_request(
                "resolve-address needs a non-empty query or enough structured address fields",
            ));
        }

        let postal_code_key = input
            .postal_code
            .as_deref()
            .and_then(non_empty)
            .map(compact_alphanumeric)
            .or_else(|| infer_postal_key_from_query(&query, country_code.as_deref()))
            .filter(|value| value.len() >= 4);

        let street = input
            .street
            .as_deref()
            .and_then(non_empty)
            .map(normalize_text)
            .or_else(|| {
                // Try to infer street-like token from query (first 2-3 words)
                let parts = query.split_whitespace().collect::<Vec<_>>();
                if parts.len() >= 2 {
                    Some(parts[..2].join(" "))
                } else {
                    None
                }
            })
            .filter(|value| value.len() >= 3);

        let city = input
            .city
            .as_deref()
            .and_then(non_empty)
            .map(normalize_text)
            .or_else(|| {
                // Try to infer city-like token from query (words near the end)
                let parts = query.split_whitespace().collect::<Vec<_>>();
                if parts.len() >= 3 {
                    // Usually city is before postal code/country
                    let mut end = parts.len();
                    if country_code.is_some() { end -= 1; }
                    if postal_code_key.is_some() { end -= 1; }
                    if end >= 1 {
                        Some(parts[end-1].to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter(|value| value.len() >= 3);
        
        let house_number_key = input
            .house_number
            .as_deref()
            .and_then(non_empty)
            .map(compact_alphanumeric)
            .filter(|value| !value.is_empty());

        let max_edit_distance = ((query.len() as f64) * 0.18).ceil() as i32;

        let fuzzy_query = if explicit_country_code.is_none() && input.postal_code.is_none() {
            // If input is purely a plain query, try to strip the inferred parts for the GIN search
            let mut parts = query.split_whitespace().collect::<Vec<_>>();
            let initial_len = parts.len();

            // Strip country from end
            if let Some(cc) = &country_code {
                if let Some(last) = parts.last() {
                    if last.eq_ignore_ascii_case(cc)
                        || canonical_country_code(last).as_deref() == Some(cc)
                    {
                        parts.pop();
                    }
                }
            }

            // Strip postal code from end
            if let Some(pc) = &postal_code_key {
                if let Some(last) = parts.last() {
                    if compact_alphanumeric(last) == *pc {
                        parts.pop();
                    } else if parts.len() >= 2 {
                        // Check two-part postal codes
                        let last_two = parts[parts.len() - 2..].join("");
                        if compact_alphanumeric(&last_two) == *pc {
                            parts.pop();
                            parts.pop();
                        }
                    }
                }
            }

            if parts.len() < initial_len && parts.len() >= 2 {
                Some(parts.join(" "))
            } else {
                Some(query.clone())
            }
        } else {
            Some(query.clone())
        };

        Ok(AddressSearch {
            query,
            fuzzy_query,
            country_code,
            postal_code_key,
            street,
            city,
            house_number_key,
            candidate_limit,
            max_edit_distance: max_edit_distance.clamp(2, 24),
        })
    }
}

impl ResolvedInput {
    fn from_request(request: ResolveAddressRequest) -> Self {
        let nested = request.address.unwrap_or_default();

        Self {
            query: request.query,
            house_number: first_present(request.house_number, nested.house_number),
            street: first_present(request.street, nested.street),
            city: first_present(request.city, nested.city),
            postal_code: first_present(request.postal_code, nested.postal_code),
            country: first_present(request.country, nested.country),
        }
    }

    fn structured_query_text(&self, country_code: Option<&str>) -> String {
        [
            self.street.as_deref(),
            self.house_number.as_deref(),
            self.city.as_deref(),
            self.postal_code.as_deref(),
            country_code,
        ]
        .into_iter()
        .flatten()
        .filter_map(non_empty)
        .collect::<Vec<_>>()
        .join(" ")
    }
}

impl Default for StructuredAddressInput {
    fn default() -> Self {
        Self {
            house_number: None,
            street: None,
            city: None,
            postal_code: None,
            country: None,
        }
    }
}

fn first_present(primary: Option<String>, fallback: Option<String>) -> Option<String> {
    primary
        .and_then(|value| (!value.trim().is_empty()).then_some(value))
        .or_else(|| fallback.and_then(|value| (!value.trim().is_empty()).then_some(value)))
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn infer_country_code_from_query(query: &str) -> Option<String> {
    let last = query.split_whitespace().last()?;
    canonical_country_code(last)
}

fn infer_postal_key_from_query(query: &str, country_code: Option<&str>) -> Option<String> {
    let tokens = query.split_whitespace().collect::<Vec<_>>();
    let mut end = tokens.len();

    // If we have an identified country code, check if the last token matches it or maps to it
    if let Some(cc) = country_code {
        if let Some(last) = tokens.last() {
            if last.eq_ignore_ascii_case(cc) || canonical_country_code(last).as_deref() == Some(cc) {
                end = end.saturating_sub(1);
            }
        }
    }

    for width in 1..=2 {
        if end < width {
            continue;
        }

        let candidate = tokens[end - width..end].join("");
        let compact = compact_alphanumeric(&candidate);
        if compact.len() >= 4
            && compact.len() <= 10
            && compact.chars().any(|c| c.is_ascii_digit())
        {
            return Some(compact);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepares_plain_text_query() {
        let search = AddressService::prepare_search_with_limit(
            ResolvedInput {
                query: Some("Avenue de France 123, Stiring-Wendel 57350 FR".to_string()),
                ..ResolvedInput::default()
            },
            250,
        )
        .expect("search input");

        assert_eq!(search.query, "avenue de france 123 stiring wendel 57350 fr");
        assert_eq!(search.country_code.as_deref(), Some("FR"));
        assert_eq!(search.postal_code_key.as_deref(), Some("57350"));
    }

    #[test]
    fn prepares_structured_query_with_filters() {
        let search = AddressService::prepare_search_with_limit(
            ResolvedInput {
                house_number: Some("221B".to_string()),
                street: Some("Baker Street".to_string()),
                city: Some("London".to_string()),
                postal_code: Some("NW1 6XE".to_string()),
                country: Some("gb".to_string()),
                ..ResolvedInput::default()
            },
            250,
        )
        .expect("search input");

        assert_eq!(search.country_code.as_deref(), Some("GB"));
        assert_eq!(search.postal_code_key.as_deref(), Some("nw16xe"));
        assert_eq!(search.house_number_key.as_deref(), Some("221b"));
        assert_eq!(search.query, "baker street 221b london nw1 6xe gb");
    }

    #[test]
    fn accepts_country_names_for_structured_country() {
        let search = AddressService::prepare_search_with_limit(
            ResolvedInput {
                query: Some("221B Baker Street London".to_string()),
                country: Some("United Kingdom".to_string()),
                ..ResolvedInput::default()
            },
            250,
        )
        .expect("valid country name should succeed");

        assert_eq!(search.country_code.as_deref(), Some("GB"));
    }

    #[test]
    fn prepares_czech_query_with_country_name() {
        let search = AddressService::prepare_search_with_limit(
            ResolvedInput {
                query: Some("Gen Svobody 174 Novy Bor 47301 Cesko".to_string()),
                ..ResolvedInput::default()
            },
            250,
        )
        .expect("search input");

        assert_eq!(search.country_code.as_deref(), Some("CZ"));
        assert_eq!(search.postal_code_key.as_deref(), Some("47301"));
        assert_eq!(search.query, "gen svobody 174 novy bor 47301 cesko");
    }

    #[test]
    fn prepares_czech_query_without_country_name() {
        let search = AddressService::prepare_search_with_limit(
            ResolvedInput {
                query: Some("Gen Svobody 174 Novy Bor 47301".to_string()),
                ..ResolvedInput::default()
            },
            250,
        )
        .expect("search input");

        // Country won't be inferred because 47301 is not a country code
        assert_eq!(search.country_code, None);
        assert_eq!(search.postal_code_key.as_deref(), Some("47301"));
    }
}
