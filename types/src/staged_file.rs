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

use teaclave_crypto::TeaclaveFile128Key;

use std::collections::HashMap;
#[cfg(not(feature = "mesalock_sgx"))]
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs::File;

use crate::FileAuthTag;
use crate::FileCrypto;
use anyhow::Context;
use protected_fs::ProtectedFile;

#[derive(Clone, Debug, Default)]
pub struct StagedFileInfo {
    pub path: PathBuf,
    pub crypto_info: TeaclaveFile128Key,
    pub cmac: FileAuthTag,
}

impl StagedFileInfo {
    pub fn new(
        path: impl AsRef<Path>,
        crypto_info: TeaclaveFile128Key,
        cmac: impl Into<FileAuthTag>,
    ) -> Self {
        StagedFileInfo {
            path: path.as_ref().into(),
            crypto_info,
            cmac: cmac.into(),
        }
    }

    pub fn create_readable_io(&self) -> anyhow::Result<Box<dyn io::Read>> {
        let f = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        let tag = f
            .current_meta_gmac()
            .context("Failed to get gmac from protected file")?;
        anyhow::ensure!(self.cmac == tag, "Corrupted input file: {:?}", self.path);
        Ok(Box::new(f))
    }

    pub fn create_writable_io(&self) -> anyhow::Result<Box<dyn io::Write>> {
        let f = ProtectedFile::create_ex(&self.path, &self.crypto_info.key)?;
        Ok(Box::new(f))
    }

    pub fn convert_for_uploading(
        &self,
        dst: impl AsRef<Path>,
        crypto_info: FileCrypto,
    ) -> anyhow::Result<FileAuthTag> {
        match crypto_info {
            FileCrypto::TeaclaveFile128(cipher) => {
                self.convert_to_teaclave_file(dst, cipher.to_owned())
            }
            FileCrypto::AesGcm128(cipher) => {
                let mut src_file = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)
                    .context("Convert aes-gcm-128: failed to open src file")?;
                let mut buffer = Vec::new();
                src_file.read_to_end(&mut buffer)?;
                let cmac = cipher.encrypt(&mut buffer)?;
                let mut file = File::create(dst)?;
                file.write_all(&buffer)?;
                FileAuthTag::from_bytes(&cmac)
            }
            FileCrypto::AesGcm256(cipher) => {
                let mut src_file = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)
                    .context("Convert aes-gcm-256: failed to open src file")?;
                let mut buffer = Vec::new();
                src_file.read_to_end(&mut buffer)?;
                let cmac = cipher.encrypt(&mut buffer)?;
                let mut file = File::create(dst)?;
                file.write_all(&buffer)?;
                FileAuthTag::from_bytes(&cmac)
            }
            FileCrypto::Raw => anyhow::bail!("OutputFile: unsupported type"),
        }
    }

    pub fn convert_to_teaclave_file(
        &self,
        dst: impl AsRef<Path>,
        crypto: TeaclaveFile128Key,
    ) -> anyhow::Result<FileAuthTag> {
        let src_file = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)
            .context("Convert teaclave_file: failed to open src file")?;
        let mut dest_file = ProtectedFile::create_ex(dst.as_ref(), &crypto.key)
            .context("Convert teaclave_file: failed to create dst file")?;

        let mut reader = BufReader::with_capacity(4096, src_file);
        loop {
            let buffer = reader.fill_buf()?;
            let rd_len = buffer.len();
            if rd_len == 0 {
                break;
            }
            let wt_len = dest_file.write(buffer)?;
            anyhow::ensure!(
                rd_len == wt_len,
                "Cannot fully write to dest file: Rd({:?}) != Wt({:?})",
                rd_len,
                wt_len
            );
            reader.consume(rd_len);
        }
        dest_file
            .flush()
            .context("Convert teaclave_file: dst_file flush failed")?;
        let mac = dest_file
            .current_meta_gmac()
            .context("Convert teaclave_file: cannot get dst_file gmac")?;
        FileAuthTag::from_bytes(&mac)
    }

    #[cfg(test_mode)]
    pub fn create_with_plaintext_file(path: impl AsRef<Path>) -> anyhow::Result<StagedFileInfo> {
        let bytes = read_all_bytes(path.as_ref())?;
        let dst = path.as_ref().with_extension("test_enc");
        Self::create_with_bytes(dst, &bytes)
    }

    #[cfg(test_mode)]
    pub fn get_plaintext(&self) -> anyhow::Result<Vec<u8>> {
        let mut content = Vec::new();
        let mut f = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        f.read_to_end(&mut content)?;
        Ok(content)
    }

    pub fn create_with_bytes(
        path: impl AsRef<Path>,
        bytes: &[u8],
    ) -> anyhow::Result<StagedFileInfo> {
        let crypto = TeaclaveFile128Key::random();
        let mut f = ProtectedFile::create_ex(&path, &crypto.key)?;
        f.write_all(bytes)?;
        f.flush()?;
        let tag = f.current_meta_gmac()?;
        Ok(Self::new(path.as_ref(), crypto, tag))
    }
}

pub fn read_all_bytes(path: impl AsRef<Path>) -> anyhow::Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut content)?;
    Ok(content)
}

#[derive(Debug, Default, Clone)]
pub struct StagedFiles {
    entries: HashMap<String, StagedFileInfo>,
}

impl StagedFiles {
    pub fn new(entries: HashMap<String, StagedFileInfo>) -> Self {
        StagedFiles { entries }
    }

    pub fn get(&self, key: &str) -> Option<&StagedFileInfo> {
        self.entries.get(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::iter::FromIterator<(String, StagedFileInfo)> for StagedFiles {
    fn from_iter<T: IntoIterator<Item = (String, StagedFileInfo)>>(iter: T) -> Self {
        StagedFiles {
            entries: HashMap::from_iter(iter),
        }
    }
}
