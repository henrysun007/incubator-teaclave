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

//! This crate provides TLS-based remote attestation mechanism for Teaclave,
//! supporting both EPID and ECDSA attestation. By default, Intel Attestation
//! Service is used for RA.

#![allow(clippy::nonstandard_macro_braces)]
#![allow(clippy::unknown_clippy_lints)]

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Errors that can happen during attestation and verification process
#[derive(thiserror::Error, Debug)]
pub enum AttestationError {
    #[error("OCall error")]
    OCallError(sgx_types::sgx_status_t),
    #[error("Attestation Service error")]
    AttestationServiceError,
    #[error("Platform error")]
    PlatformError(sgx_types::sgx_status_t),
    #[error("Report error")]
    ReportError,
    #[error("Report error")]
    ConnectionError,
    #[error("Attestation Service API version not compatible")]
    ApiVersionNotCompatible,
}

/// Remote attestation configuration
#[derive(Clone)]
pub enum AttestationConfig {
    /// Trust enclave without attestation
    NoAttestation,
    /// Perform attestation before trusting enclave
    WithAttestation(AttestationServiceConfig),
}

/// Remote attestation algorithm
#[derive(Clone)]
pub(crate) enum AttestationAlgorithm {
    /// Use Intel EPID
    SgxEpid,
    /// Use ECDSA
    SgxEcdsa,
}

impl AttestationAlgorithm {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "sgx_epid" => Some(AttestationAlgorithm::SgxEpid),
            "sgx_ecdsa" => Some(AttestationAlgorithm::SgxEcdsa),
            _ => None,
        }
    }
}

/// Attestation Service Configuration
#[derive(Clone)]
pub struct AttestationServiceConfig {
    /// Algorithm to use
    algo: AttestationAlgorithm,
    /// URL of attestation service
    as_url: url::Url,
    /// IAS API Key
    api_key: String,
    /// SPID
    spid: sgx_types::sgx_spid_t,
}

pub struct DcapConfig {}

impl AttestationConfig {
    /// Creates `AttestationConfig` for no attestation
    pub fn no_attestation() -> Arc<Self> {
        Arc::new(Self::NoAttestation)
    }

    /// Creates `AttestationConfig` for attestation using given values
    pub fn new(algorithm: &str, url: &str, api_key: &str, spid_str: &str) -> Result<Arc<Self>> {
        if cfg!(sgx_sim) {
            return Ok(Self::no_attestation());
        }

        use core::convert::TryFrom;

        let mut spid = sgx_types::sgx_spid_t::default();
        let hex = hex::decode(spid_str).context("Illegal SPID provided")?;
        spid.id = <[u8; 16]>::try_from(hex.as_slice()).context("Illegal SPID provided")?;

        let algo = AttestationAlgorithm::from_str(algorithm)
            .context("Unsupported remote attestation algorithm")?;

        let att_service_cfg = AttestationServiceConfig {
            algo,
            as_url: url::Url::parse(url).context("Invalid URL")?,
            api_key: api_key.to_string(),
            spid,
        };

        Ok(Arc::new(Self::WithAttestation(att_service_cfg)))
    }

    /// Crate attestation config from Teaclave runtime configuration.
    pub fn from_teaclave_config(config: &teaclave_config::RuntimeConfig) -> Result<Arc<Self>> {
        let as_config = &config.attestation;
        Self::new(
            &as_config.algorithm,
            &as_config.url,
            &as_config.key,
            &as_config.spid,
        )
    }
}

/// AttestationReport can be endorsed by either the Intel Attestation Service
/// using EPID or Data Center Attestation
/// Service (platform dependent) using ECDSA.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EndorsedAttestationReport {
    /// Attestation report generated by the hardware
    pub report: Vec<u8>,
    /// Singature of the report
    pub signature: Vec<u8>,
    /// Certificate matching the signing key of the signature
    pub certs: Vec<Vec<u8>>,
}

/// Configuration for TLS communication in Remote Attestation
#[derive(Debug)]
pub struct AttestedTlsConfig {
    pub cert: Vec<u8>,
    pub private_key: Vec<u8>,
    pub time: std::time::SystemTime,
    pub validity: std::time::Duration,
}

#[macro_use]
mod cert;
pub mod report;
pub mod verifier;

cfg_if::cfg_if! {
    if #[cfg(feature = "mesalock_sgx")]  {
        mod service;
        pub mod key;
        mod platform;
        mod attestation;
        pub use attestation::RemoteAttestation;
    }
}

#[cfg(all(feature = "enclave_unit_test", feature = "mesalock_sgx"))]
pub mod tests {
    use super::*;
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_tests!(
            platform::tests::test_init_sgx_quote,
            platform::tests::test_create_sgx_isv_enclave_report,
            platform::tests::test_get_sgx_quote,
            report::tests::test_sgx_quote_parse_from,
            report::tests::test_attestation_report_from_cert,
            report::tests::test_attestation_report_from_cert_api_version_not_compatible
        )
    }
}
