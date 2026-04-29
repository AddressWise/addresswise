use sqlx::{FromRow, PgPool};

use crate::models::{Address, MatchDiagnostics, ResolveAddressResponse};

#[derive(Clone)]
pub struct AddressRepository {
    pool: PgPool,
}

#[derive(Debug)]
pub struct AddressSearch {
    pub query: String,
    pub country_code: Option<String>,
    pub postal_code_key: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub house_number_key: Option<String>,
    pub max_edit_distance: i32,
}

#[derive(Debug, FromRow)]
struct AddressCandidate {
    id: i64,
    formatted: String,
    country_code: String,
    admin_area: Option<String>,
    locality: Option<String>,
    dependent_locality: Option<String>,
    thoroughfare: Option<String>,
    premise: Option<String>,
    premise_type: Option<String>,
    subpremise: Option<String>,
    postal_code: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    score: f64,
    trigram_score: f64,
    edit_distance: i32,
}

impl AddressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_ids(
        &self,
        ids: &[i64],
        search: &AddressSearch,
    ) -> Result<Vec<ResolveAddressResponse>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AddressCandidate>(
            r#"
            WITH candidate_pool AS (
                SELECT
                    a.id, a.country_code::text, a.admin_area, a.locality, a.dependent_locality,
                    a.thoroughfare, a.premise, a.premise_type, a.subpremise, a.postal_code,
                    a.latitude, a.longitude, a.full_address AS formatted, a.search_text,
                    similarity(a.search_text, $1)::float8 AS trigram_score
                FROM addresses a
                WHERE a.id = ANY($2)
            ),
            scored_candidates AS (
                SELECT
                    *,
                    levenshtein_less_equal(
                        LEFT(search_text, 255),
                        LEFT($1, 255),
                        $3::int
                    ) AS edit_distance
                FROM candidate_pool
            )
            SELECT
                id, formatted, country_code, admin_area, locality, dependent_locality, 
                thoroughfare, premise, premise_type, subpremise, postal_code, 
                latitude, longitude, 
                LEAST(
                    1.0,
                    trigram_score * 0.62
                    + CASE WHEN $4::text IS NOT NULL AND country_code = $4 THEN 0.06 ELSE 0.0 END
                    + CASE WHEN $5::text IS NOT NULL AND public.address_wise_compact(postal_code) = $5 THEN 0.14 ELSE 0.0 END
                    + CASE WHEN $6::text IS NOT NULL AND (public.address_wise_compact(premise) = $6 OR public.address_wise_compact(premise) LIKE $6 || '%') THEN 0.12 ELSE 0.0 END
                    + CASE WHEN $7::text IS NOT NULL THEN GREATEST(similarity(search_text, $7)::float8, similarity(COALESCE(public.address_wise_normalize(thoroughfare), ''), $7)::float8) * 0.12 ELSE 0.0 END
                    + CASE WHEN $8::text IS NOT NULL THEN GREATEST(similarity(search_text, $8)::float8, similarity(COALESCE(public.address_wise_normalize(locality), ''), $8)::float8) * 0.20 ELSE 0.0 END
                    + CASE WHEN edit_distance <= $3 THEN (1.0 - (edit_distance::float8 / GREATEST(length($1), 1)::float8)) * 0.10 ELSE 0.0 END
                )::float8 AS score,
                trigram_score,
                edit_distance
            FROM scored_candidates
            "#,
        )
        .bind(&search.query)
        .bind(ids)
        .bind(search.max_edit_distance)
        .bind(search.country_code.as_deref())
        .bind(search.postal_code_key.as_deref())
        .bind(search.house_number_key.as_deref())
        .bind(search.street.as_deref())
        .bind(search.city.as_deref())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|candidate| {
                candidate.into_response(search.query.clone(), search.max_edit_distance)
            })
            .collect())
    }
}

impl AddressCandidate {
    fn into_response(self, query: String, max_edit_distance: i32) -> ResolveAddressResponse {
        ResolveAddressResponse {
            query,
            score: round_score(self.score),
            address: Address {
                id: self.id,
                formatted: self.formatted,
                country_code: self.country_code.trim().to_string(),
                admin_area: self.admin_area,
                locality: self.locality,
                dependent_locality: self.dependent_locality,
                street: self.thoroughfare,
                house_number: self.premise,
                house_number_type: self.premise_type,
                unit: self.subpremise,
                postal_code: self.postal_code,
                latitude: self.latitude,
                longitude: self.longitude,
            },
            diagnostics: MatchDiagnostics {
                trigram_score: round_score(self.trigram_score),
                levenshtein_distance: (self.edit_distance <= max_edit_distance)
                    .then_some(self.edit_distance),
            },
        }
    }
}

fn round_score(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[derive(Debug, sqlx::FromRow)]
pub struct AddressRecord {
    pub id: i64,
    pub search_text: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AutocompleteAddressRecord {
    pub id: i64,
    pub country_code: String,
    pub thoroughfare: Option<String>,
    pub full_address: String,
    pub locality: Option<String>,
    pub postal_code: Option<String>,
}

impl AddressRepository {
    pub fn stream_all(
        &self,
    ) -> impl futures::Stream<Item = Result<AddressRecord, sqlx::Error>> + '_ {
        sqlx::query_as::<_, AddressRecord>(
            "SELECT id, search_text FROM addresses WHERE is_active = TRUE",
        )
        .fetch(&self.pool)
    }

    pub fn stream_autocomplete(
        &self,
    ) -> impl futures::Stream<Item = Result<AutocompleteAddressRecord, sqlx::Error>> + '_ {
        sqlx::query_as::<_, AutocompleteAddressRecord>(
            "SELECT id, country_code::text, thoroughfare, full_address, locality, postal_code \
             FROM addresses \
             WHERE is_active = TRUE AND thoroughfare IS NOT NULL AND full_address <> ''",
        )
        .fetch(&self.pool)
    }
}
