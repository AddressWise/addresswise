CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS fuzzystrmatch;
CREATE EXTENSION IF NOT EXISTS unaccent;

CREATE TABLE IF NOT EXISTS addresses (
    id BIGSERIAL PRIMARY KEY,
    country_code CHAR(2) NOT NULL,
    source_dataset TEXT,
    admin_area TEXT,
    locality TEXT,
    dependent_locality TEXT,
    thoroughfare TEXT,
    premise TEXT,
    premise_type TEXT,
    subpremise TEXT,
    postal_code TEXT,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    source_hash TEXT,
    full_address TEXT NOT NULL,
    search_text TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_seen_run BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT country_code_uppercase CHECK (country_code = UPPER(country_code))
);

CREATE OR REPLACE FUNCTION public.address_wise_unaccent(input TEXT)
RETURNS TEXT
LANGUAGE SQL
IMMUTABLE
PARALLEL SAFE
RETURNS NULL ON NULL INPUT
AS $$
    SELECT public.unaccent('public.unaccent'::regdictionary, input)
$$;

CREATE OR REPLACE FUNCTION public.address_wise_normalize(input TEXT)
RETURNS TEXT
LANGUAGE SQL
IMMUTABLE
PARALLEL SAFE
RETURNS NULL ON NULL INPUT
AS $$
    SELECT btrim(regexp_replace(lower(public.address_wise_unaccent(input)), '[^a-z0-9]+', ' ', 'g'))
$$;

CREATE OR REPLACE FUNCTION public.address_wise_compact(input TEXT)
RETURNS TEXT
LANGUAGE SQL
IMMUTABLE
PARALLEL SAFE
RETURNS NULL ON NULL INPUT
AS $$
    SELECT regexp_replace(public.address_wise_normalize(input), '[[:space:]]+', '', 'g')
$$;

CREATE INDEX IF NOT EXISTS idx_addresses_prefix_btree
    ON addresses ((LEFT(search_text, 2)), search_text);

CREATE INDEX IF NOT EXISTS idx_addresses_search_trgm
    ON addresses USING gin (search_text gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_addresses_postal_code
    ON addresses (postal_code);

CREATE INDEX IF NOT EXISTS idx_addresses_source_dataset_active
    ON addresses (source_dataset, is_active);

CREATE UNIQUE INDEX IF NOT EXISTS uq_addresses_country_source_hash
    ON addresses (country_code, source_hash);

CREATE OR REPLACE FUNCTION public.set_updated_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_addresses_updated_at ON addresses;
CREATE TRIGGER trg_addresses_updated_at
BEFORE UPDATE ON addresses
FOR EACH ROW
EXECUTE FUNCTION public.set_updated_at();
