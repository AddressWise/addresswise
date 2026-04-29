use crate::error::AppError;
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexReader, ReloadPolicy};

#[derive(Clone)]
pub struct AddressIndex {
    pub index: Index,
    pub reader: IndexReader,
    pub fields: AddressFields,
}

#[derive(Clone)]
pub struct AddressFields {
    pub id: Field,
    pub search_text: Field,
}

impl AddressIndex {
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let mut schema_builder = Schema::builder();

        let id = schema_builder.add_u64_field("id", INDEXED | STORED);
        let search_text = schema_builder.add_text_field("search_text", TEXT);

        let schema = schema_builder.build();
        let fields = AddressFields { id, search_text };

        let index = if path.as_ref().exists() {
            Index::open_in_dir(path)?
        } else {
            std::fs::create_dir_all(&path).map_err(|e| AppError::Internal(e.into()))?;
            Index::create_in_dir(path, schema.clone())?
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            fields,
        })
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<TantivyCandidate>, AppError> {
        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.search_text]);

        // We use a fuzzy query or just a standard query parser?
        // Let's start with standard.
        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| AppError::bad_request(e.to_string()))?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            results.push(TantivyCandidate {
                id: retrieved_doc
                    .get_first(self.fields.id)
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as i64,
            });
        }

        Ok(results)
    }
}

pub struct TantivyCandidate {
    pub id: i64,
}
