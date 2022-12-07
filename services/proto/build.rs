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

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

fn main() {
    let proto_files = [
        "src/proto/teaclave_access_control_service.proto",
        "src/proto/teaclave_authentication_service.proto",
        "src/proto/teaclave_common.proto",
        "src/proto/teaclave_storage_service.proto",
        "src/proto/teaclave_frontend_service.proto",
        "src/proto/teaclave_management_service.proto",
        "src/proto/teaclave_scheduler_service.proto",
    ];

    let out_dir = env::var("OUT_DIR").expect("$OUT_DIR not set. Please build with cargo");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto_gen/templates/proto.j2");
    println!("cargo:rerun-if-changed=proto_gen/main.rs");

    for pf in proto_files.iter() {
        println!("cargo:rerun-if-changed={}", pf);
    }

    let target_dir = match env::var("TEACLAVE_SYMLINKS") {
        Ok(teaclave_symlinks) => {
            Path::new(&teaclave_symlinks).join("teaclave_build/target/proto_gen")
        }
        Err(_) => env::current_dir().unwrap().join("target/proto_gen"),
    };
    let current_dir: PathBuf = match env::var("MT_SGXAPP_TOML_DIR") {
        Ok(sgxapp_toml_dir) => Path::new(&sgxapp_toml_dir).into(),
        // This fallback is only for compiling rust client sdk with cargo
        Err(_) => Path::new("../../").into(),
    };

    // Use the offline flag when building within Teaclave's build system.
    let _offline = env::var("MT_SGXAPP_TOML_DIR").is_ok();

    let proto_files = [
        "services/proto/src/proto/teaclave_access_control_service.proto",
        "services/proto/src/proto/teaclave_authentication_service.proto",
        "services/proto/src/proto/teaclave_common.proto",
        "services/proto/src/proto/teaclave_storage_service.proto",
        "services/proto/src/proto/teaclave_frontend_service.proto",
        "services/proto/src/proto/teaclave_management_service.proto",
        "services/proto/src/proto/teaclave_scheduler_service.proto",
    ];
    let mut c = Command::new("cargo");
    // Use CARGO_ENCODED_RUSTFLAGS to override RUSTFLAGS which makes the run fail.
    c.current_dir(&current_dir)
        .env("CARGO_ENCODED_RUSTFLAGS", "");
    c.args([
        "run",
        "--target-dir",
        &target_dir.to_string_lossy(),
        "--manifest-path",
        "services/proto/proto_gen/Cargo.toml",
    ]);

    c.args(["--", "-i", "services/proto/src/proto", "-d", &out_dir, "-p"])
        .args(proto_files);
    let output = c.output().expect("Generate proto failed");
    if !output.status.success() {
        panic!(
            "stdout: {:?}, stderr: {:?}",
            str::from_utf8(&output.stderr).unwrap(),
            str::from_utf8(&output.stderr).unwrap()
        );
    }
}
