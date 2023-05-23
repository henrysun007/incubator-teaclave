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

use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    microsecond: i64,
    ip: Ipv4Addr,
    user: String,
    message: String,
    result: bool,
}

impl Default for Entry {
    fn default() -> Self {
        let microsecond = 0;
        let ip = Ipv4Addr::UNSPECIFIED;
        let user = String::new();
        let message = String::new();
        let result = false;

        Self {
            microsecond,
            ip,
            user,
            message,
            result,
        }
    }
}

impl Entry {
    pub fn microsecond(&self) -> i64 {
        self.microsecond
    }

    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn user(&self) -> String {
        self.user.clone()
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }

    pub fn result(&self) -> bool {
        self.result
    }
}

#[derive(Default, Clone)]
pub struct EntryBuilder {
    microsecond: Option<i64>,
    ip: Option<Ipv4Addr>,
    user: Option<String>,
    message: Option<String>,
    result: Option<bool>,
}

impl EntryBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn microsecond(mut self, microsecond: i64) -> Self {
        self.microsecond = Some(microsecond);
        self
    }

    pub fn ip(mut self, ip: Ipv4Addr) -> Self {
        self.ip = Some(ip);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn result(mut self, result: bool) -> Self {
        self.result = Some(result);
        self
    }

    pub fn build(self) -> Entry {
        let microsecond = self.microsecond.unwrap_or_else(|| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let microsecond = now.as_micros() as i64;
            microsecond
        });

        Entry {
            microsecond,
            ip: self.ip.unwrap_or(Ipv4Addr::UNSPECIFIED),
            user: self.user.unwrap_or_default(),
            message: self.message.unwrap_or_default(),
            result: self.result.unwrap_or(false),
        }
    }
}
