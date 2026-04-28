use sqlx::{FromRow, PgPool};

use crate::models::{Address, MatchDiagnostics, ResolveAddressResponse};

#[derive(Clone)]
pub struct AddressRepository {
    pool: PgPool,
}

#[derive(Debug)]
pub struct AddressSearch {
    pub query: String,
    pub fuzzy_query: Option<String>,
    pub country_code: Option<String>,
    pub postal_code_key: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub house_number_key: Option<String>,
    pub candidate_limit: i64,
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

        Ok(rows.into_iter().map(|candidate| {
            candidate.into_response(search.query.clone(), search.max_edit_distance)
        }).collect())
    }

    pub async fn resolve(
        &self,
        search: &AddressSearch,
    ) -> Result<Option<ResolveAddressResponse>, sqlx::Error> {
        let mut query_builder = sqlx::QueryBuilder::new("WITH candidate_pool AS (");
        let mut has_branch = false;

        // 1. High priority: Exact postal code match (B-tree)
        if let Some(pc) = &search.postal_code_key {
            query_builder.push("(SELECT a.id, a.country_code::text, a.admin_area, a.locality, a.dependent_locality, a.thoroughfare, a.premise, a.premise_type, a.subpremise, a.postal_code, a.latitude, a.longitude, a.full_address AS formatted, a.search_text ");
            query_builder.push("FROM addresses a WHERE a.is_active = TRUE ");
            query_builder.push("AND public.address_wise_compact(a.postal_code) = ").push_bind(pc);
            if let Some(cc) = &search.country_code {
                query_builder.push(" AND a.country_code = ").push_bind(cc);
            }
            query_builder.push(" LIMIT ").push_bind(search.candidate_limit).push(")");
            has_branch = true;
        }

        // 2. Medium priority: Specific fuzzy query (GIN)
        if let Some(fuzzy) = &search.fuzzy_query {
            if has_branch { query_builder.push(" UNION ALL "); }
            query_builder.push("(SELECT a.id, a.country_code::text, a.admin_area, a.locality, a.dependent_locality, a.thoroughfare, a.premise, a.premise_type, a.subpremise, a.postal_code, a.latitude, a.longitude, a.full_address AS formatted, a.search_text ");
            query_builder.push("FROM addresses a WHERE a.is_active = TRUE AND a.search_text % ").push_bind(fuzzy);
            if let Some(cc) = &search.country_code {
                query_builder.push(" AND a.country_code = ").push_bind(cc);
            }
            if let Some(pc) = &search.postal_code_key {
                query_builder.push(" AND public.address_wise_compact(a.postal_code) != ").push_bind(pc);
            }
            query_builder.push(" LIMIT ").push_bind(search.candidate_limit).push(")");
            has_branch = true;
        }

        // 3. Fallback: Street AND City
        if let (Some(street), Some(city)) = (&search.street, &search.city) {
            if has_branch { query_builder.push(" UNION ALL "); }
            query_builder.push("(SELECT a.id, a.country_code::text, a.admin_area, a.locality, a.dependent_locality, a.thoroughfare, a.premise, a.premise_type, a.subpremise, a.postal_code, a.latitude, a.longitude, a.full_address AS formatted, a.search_text ");
            query_builder.push("FROM addresses a WHERE a.is_active = TRUE ");
            query_builder.push("AND a.search_text % ").push_bind(street);
            query_builder.push(" AND a.search_text % ").push_bind(city);
            if let Some(cc) = &search.country_code {
                query_builder.push(" AND a.country_code = ").push_bind(cc);
            }
            if let Some(pc) = &search.postal_code_key {
                query_builder.push(" AND public.address_wise_compact(a.postal_code) != ").push_bind(pc);
            }
            query_builder.push(" LIMIT 100)");
            has_branch = true;
        }

        if !has_branch {
            query_builder.push("(SELECT a.id, a.country_code::text, a.admin_area, a.locality, a.dependent_locality, a.thoroughfare, a.premise, a.premise_type, a.subpremise, a.postal_code, a.latitude, a.longitude, a.full_address AS formatted, a.search_text ");
            query_builder.push("FROM addresses a WHERE a.is_active = TRUE AND a.search_text % ").push_bind(&search.query);
            if let Some(cc) = &search.country_code {
                query_builder.push(" AND a.country_code = ").push_bind(cc);
            }
            query_builder.push(" LIMIT ").push_bind(search.candidate_limit).push(")");
        }

        query_builder.push("), scored_candidates AS (");
        query_builder.push("SELECT *, ");
        query_builder.push("similarity(search_text, ").push_bind(&search.query).push(")::float8 AS trigram_score, ");
        query_builder.push("levenshtein_less_equal(LEFT(search_text, 255), LEFT(").push_bind(&search.query).push(", 255), ").push_bind(search.max_edit_distance).push(") AS edit_distance ");
        query_builder.push("FROM candidate_pool) ");
        
        query_builder.push("SELECT id, formatted, country_code, admin_area, locality, dependent_locality, thoroughfare, premise, premise_type, subpremise, postal_code, latitude, longitude, LEAST(1.0, trigram_score * 0.62 + ");
        
        query_builder.push("CASE WHEN ");
        if let Some(cc) = &search.country_code {
            query_builder.push("country_code = ").push_bind(cc).push(" THEN 0.06 ELSE 0.0 END");
        } else {
            query_builder.push("FALSE THEN 0.06 ELSE 0.0 END");
        }
        
        query_builder.push(" + CASE WHEN ");
        if let Some(pc) = &search.postal_code_key {
            query_builder.push("public.address_wise_compact(postal_code) = ").push_bind(pc).push(" THEN 0.14 ELSE 0.0 END");
        } else {
            query_builder.push("FALSE THEN 0.14 ELSE 0.0 END");
        }

        query_builder.push(" + CASE WHEN ");
        if let Some(hn) = &search.house_number_key {
            query_builder.push("(public.address_wise_compact(premise) = ").push_bind(hn).push(" OR public.address_wise_compact(premise) LIKE ").push_bind(format!("{}%", hn)).push(") THEN 0.12 ELSE 0.0 END");
        } else {
            query_builder.push("FALSE THEN 0.12 ELSE 0.0 END");
        }

        query_builder.push(" + CASE WHEN ");
        if let Some(st) = &search.street {
            query_builder.push("TRUE THEN GREATEST(similarity(search_text, ").push_bind(st).push(")::float8, similarity(COALESCE(public.address_wise_normalize(thoroughfare), ''), ").push_bind(st).push(")::float8) * 0.12 ELSE 0.0 END");
        } else {
            query_builder.push("FALSE THEN 0.0 ELSE 0.0 END");
        }

        query_builder.push(" + CASE WHEN ");
        if let Some(ct) = &search.city {
            query_builder.push("TRUE THEN GREATEST(similarity(search_text, ").push_bind(ct).push(")::float8, similarity(COALESCE(public.address_wise_normalize(locality), ''), ").push_bind(ct).push(")::float8) * 0.08 ELSE 0.0 END");
        } else {
            query_builder.push("FALSE THEN 0.0 ELSE 0.0 END");
        }

        query_builder.push(" + CASE WHEN edit_distance <= ");
        query_builder.push_bind(search.max_edit_distance);
        query_builder.push(" THEN (1.0 - (edit_distance::float8 / GREATEST(length(");
        query_builder.push_bind(&search.query);
        query_builder.push("), 1)::float8)) * 0.10 ELSE 0.0 END)::float8 AS score, trigram_score, edit_distance FROM scored_candidates ORDER BY score DESC, trigram_score DESC, edit_distance ASC, id ASC LIMIT 1");

        let row = query_builder
            .build_query_as::<AddressCandidate>()
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|candidate| {
            candidate.into_response(search.query.clone(), search.max_edit_distance)
        }))
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

impl AddressRepository {
    pub fn stream_all(&self) -> impl futures::Stream<Item = Result<AddressRecord, sqlx::Error>> + '_ {
        sqlx::query_as::<_, AddressRecord>(
            "SELECT id, search_text FROM addresses WHERE is_active = TRUE"
        ).fetch(&self.pool)
    }
}
