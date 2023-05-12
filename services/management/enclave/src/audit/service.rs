// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use super::*;

use teaclave_proto::teaclave_storage_service_proto::TeaclaveStorageClient;

use std::convert::{From, TryFrom};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use tantivy::{
    collector::TopDocs, query::QueryParser, schema::*, DateTime, Index, IndexReader, IndexSettings,
    IndexSortByField, IndexWriter, Order, ReloadPolicy,
};

#[derive(Clone)]
pub struct LogService {
    index: Arc<Mutex<Index>>,
    reader: Arc<Mutex<IndexReader>>,
    writer: Arc<Mutex<IndexWriter>>,
}

impl LogService {
    pub fn try_new(storage: Arc<Mutex<TeaclaveStorageClient>>) -> Result<Self> {
        let directory = db_directory::DbDirectory::new(storage);

        let schema = Self::schema();

        let settings = IndexSettings {
            sort_by_field: Some(IndexSortByField {
                field: "date".to_string(),
                order: Order::Desc,
            }),
            ..Default::default()
        };

        let index = Index::builder()
            .schema(schema)
            .settings(settings)
            .open_or_create(directory)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;
        let writer = index.writer_with_num_threads(1, 8 * 3_000_000)?;

        let index = Arc::new(Mutex::new(index));
        let reader = Arc::new(Mutex::new(reader));
        let writer = Arc::new(Mutex::new(writer));

        Ok(Self {
            index,
            reader,
            writer,
        })
    }

    pub fn schema() -> Schema {
        let mut builder = Schema::builder();
        builder.add_date_field("date", INDEXED | FAST | STORED);
        builder.add_ip_addr_field("ip", INDEXED | STORED);
        builder.add_text_field("user", TEXT | STORED);
        builder.add_text_field("message", TEXT | STORED);
        builder.add_bool_field("result", INDEXED | STORED);
        let schema = builder.build();

        schema
    }

    pub fn add_log(&self, log: Entry) -> Result<()> {
        let document = Document::from(log);

        let mut writer = self.writer.lock().unwrap();
        writer.add_document(document)?;
        writer.commit()?;

        Ok(())
    }

    pub fn query(&self, query: &str, limit: usize) -> Result<Vec<Entry>> {
        let index = self.index.lock().unwrap();
        let schema = Self::schema();

        let reader = self.reader.lock().unwrap();
        let searcher = reader.searcher();

        let message = schema.get_field("message").unwrap();
        let date = schema.get_field("date").unwrap();

        let query_parser = QueryParser::for_index(&index, vec![message]);
        let query = query_parser.parse_query(query)?;

        let top_docs = searcher.search(
            &query,
            &TopDocs::with_limit(limit).order_by_fast_field::<DateTime>(date),
        )?;

        let mut entries = Vec::new();

        for (_, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let entry = Entry::try_from(retrieved_doc)?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    date: DateTime,
    ip: Ipv6Addr,
    user: String,
    message: String,
    result: bool,
}

impl Default for Entry {
    fn default() -> Self {
        let date = DateTime::from_timestamp_micros(0);
        let ip = Ipv6Addr::UNSPECIFIED;
        let user = String::new();
        let message = String::new();
        let result = false;

        Self {
            date,
            ip,
            user,
            message,
            result,
        }
    }
}

#[derive(Default)]
pub struct EntryBuilder {
    date: Option<DateTime>,
    ip: Option<Ipv6Addr>,
    user: Option<String>,
    message: Option<String>,
    result: Option<bool>,
}

impl EntryBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn date(mut self, micros: i64) -> Self {
        let date = DateTime::from_timestamp_micros(micros);
        self.date = Some(date);
        self
    }

    pub fn ip(mut self, ip: Ipv4Addr) -> Self {
        let ip = ip.to_ipv6_compatible();
        self.ip = Some(ip);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn message(mut self, message: &str) -> Self {
        self.message = Some(message.to_owned());
        self
    }

    pub fn result(mut self, result: bool) -> Self {
        self.result = Some(result);
        self
    }

    pub fn build(mut self) -> Entry {
        let date = self.date.unwrap_or_else(|| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let micros = now.as_micros() as i64;
            DateTime::from_timestamp_micros(micros)
        });

        Entry {
            date,
            ip: self.ip.unwrap_or(Ipv6Addr::UNSPECIFIED),
            user: self.user.unwrap_or_default(),
            message: self.message.unwrap_or_default(),
            result: self.result.unwrap_or(false),
        }
    }
}

impl TryFrom<Document> for Entry {
    type Error = anyhow::Error;

    fn try_from(doc: Document) -> Result<Self, Self::Error> {
        let schema = LogService::schema();
        let date = schema.get_field("date").unwrap();
        let ip = schema.get_field("ip").unwrap();
        let user = schema.get_field("user").unwrap();
        let message = schema.get_field("message").unwrap();
        let result = schema.get_field("result").unwrap();

        let date = doc
            .get_first(date)
            .and_then(|d| d.as_date())
            .ok_or_else(|| anyhow!("failed to get date"))?;
        let ip = doc
            .get_first(ip)
            .and_then(|i| i.as_ip_addr())
            .ok_or_else(|| anyhow!("failed to get ip"))?;
        let user = doc
            .get_first(user)
            .and_then(|u| u.as_text())
            .ok_or_else(|| anyhow!("failed to get user"))?;
        let message = doc
            .get_first(message)
            .and_then(|m| m.as_text())
            .ok_or_else(|| anyhow!("failed to get message"))?;
        let result = doc
            .get_first(result)
            .and_then(|r| r.as_bool())
            .ok_or_else(|| anyhow!("failed to get result"))?;

        Ok(Self {
            date,
            ip,
            user: user.to_owned(),
            message: message.to_owned(),
            result,
        })
    }
}

impl From<Entry> for Document {
    fn from(entry: Entry) -> Self {
        let schema = LogService::schema();
        let date = schema.get_field("date").unwrap();
        let ip = schema.get_field("ip").unwrap();
        let user = schema.get_field("user").unwrap();
        let message = schema.get_field("message").unwrap();
        let result = schema.get_field("result").unwrap();

        let mut doc = Document::default();
        doc.add_date(date, entry.date);
        doc.add_ip_addr(ip, entry.ip);
        doc.add_text(user, &entry.user);
        doc.add_text(message, &entry.message);
        doc.add_bool(result, entry.result);

        doc
    }
}
