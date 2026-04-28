-- Drop the old GiST index if it exists (it's slow for this dataset)
DROP INDEX IF EXISTS idx_addresses_search_gist;

-- Add GIN index for trigram similarity support
-- This is generally faster for % operator on large datasets than GiST
CREATE INDEX IF NOT EXISTS idx_addresses_search_trgm_gin
    ON addresses USING gin (search_text gin_trgm_ops);

-- Add functional index for compact postal code matching
CREATE INDEX IF NOT EXISTS idx_addresses_postal_code_compact
    ON addresses (public.address_wise_compact(postal_code));

-- Add index for country_code + is_active to speed up filtered searches
CREATE INDEX IF NOT EXISTS idx_addresses_country_active
    ON addresses (country_code, is_active);

-- Analyze the table to update statistics
ANALYZE addresses;
