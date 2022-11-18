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

use crate::utils::*;
use std::convert::TryFrom;
use teaclave_proto::teaclave_common::*;
use teaclave_proto::teaclave_common::{ExecutorCommand, ExecutorStatus};
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_test_utils::test_case;
use teaclave_types::*;
use url::Url;
use uuid::Uuid;

fn authorized_client() -> TeaclaveFrontendClient {
    let mut api_client =
        create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR).unwrap();
    let cred = login(&mut api_client, USERNAME, TEST_PASSWORD).unwrap();
    create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred).unwrap()
}

fn unauthorized_client() -> TeaclaveFrontendClient {
    let cred = UserCredential::new(USERNAME, "InvalidToken");
    create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred).unwrap()
}

#[test_case]
fn test_register_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let request = RegisterInputFileRequest::new(url.clone(), cmac, crypto_info);
    let response = authorized_client().register_input_file(request);
    assert!(response.is_ok());

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = unauthorized_client().register_input_file(request);
    assert!(response.is_err());
}

#[test_case]
fn test_update_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = authorized_client().register_input_file(request);
    assert!(response.is_ok());

    let old_data_id = response.unwrap().data_id;
    let new_url = Url::parse("https://external-storage.com/filepath-new?presigned_token").unwrap();
    let update_request = UpdateInputFileRequest::new(old_data_id.clone(), new_url);
    let update_response = authorized_client().update_input_file(update_request);
    assert!(update_response.is_ok());
    assert!(old_data_id != update_response.unwrap().data_id);
}

#[test_case]
fn test_register_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();

    let request = RegisterOutputFileRequest::new(url.clone(), crypto_info);
    let response = authorized_client().register_output_file(request);
    assert!(response.is_ok());

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = unauthorized_client().register_output_file(request);
    assert!(response.is_err());
}

#[test_case]
fn test_update_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = authorized_client().register_output_file(request);
    assert!(response.is_ok());

    let old_data_id = response.unwrap().data_id;
    let new_url = Url::parse("https://external-storage.com/filepath-new?presigned_token").unwrap();
    let update_request = UpdateOutputFileRequest::new(old_data_id.clone(), new_url);
    let update_response = authorized_client().update_output_file(update_request);
    assert!(update_response.is_ok());
    assert!(old_data_id != update_response.unwrap().data_id);
}

#[test_case]
fn test_register_fusion_output() {
    let request = RegisterFusionOutputRequest::new(vec!["frontend_user", "mock_user"]);
    let response = authorized_client().register_fusion_output(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["frontend_user", "mock_user"]);
    let response = unauthorized_client().register_fusion_output(request);
    assert!(response.is_err());
}

#[test_case]
fn test_register_input_from_output() {
    let output_id = ExternalID::try_from("output-00000000-0000-0000-0000-000000000001").unwrap();

    let request = RegisterInputFromOutputRequest::new(output_id.clone());
    let response = authorized_client().register_input_from_output(request);
    assert!(response.is_ok());

    let request = RegisterInputFromOutputRequest::new(output_id);
    let response = unauthorized_client().register_input_from_output(request);
    assert!(response.is_err());
}

#[test_case]
fn test_get_output_file() {
    let mut client = authorized_client();

    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = client.register_output_file(request).unwrap();
    let data_id = response.data_id;

    let request = GetOutputFileRequest::new(data_id.clone());
    client.get_output_file(request).unwrap();

    let request = GetOutputFileRequest::new(data_id);
    let response = unauthorized_client().get_output_file(request);
    assert!(response.is_err());
}

#[test_case]
fn test_get_input_file() {
    let mut client = authorized_client();

    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = client.register_input_file(request).unwrap();
    let data_id = response.data_id;

    let request = GetInputFileRequest::new(data_id.clone());
    client.get_input_file(request).unwrap();

    let request = GetInputFileRequest::new(data_id);
    let response = unauthorized_client().get_input_file(request);
    assert!(response.is_err());
}

#[test_case]
fn test_register_function() {
    let request = RegisterFunctionRequest::default();
    let response = authorized_client().register_function(request);
    assert!(response.is_ok());

    let request = RegisterFunctionRequest::default();
    let response = unauthorized_client().register_function(request);
    assert!(response.is_err());
}

#[test_case]
fn test_get_function() {
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000001").unwrap();

    let request = GetFunctionRequest::new(function_id.clone());
    let response = authorized_client().get_function(request);
    assert!(response.is_ok());

    let request = GetFunctionRequest::new(function_id);
    let response = unauthorized_client().get_function(request);
    assert!(response.is_err());
}

#[test_case]
fn test_create_task() {
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();

    let request = CreateTaskRequest::new()
        .function_id(function_id.clone())
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" =>  vec!["frontend_user", "mock_user"]));
    let response = authorized_client().create_task(request);
    assert!(response.is_ok());

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user", "mock_user"]));
    let response = unauthorized_client().create_task(request);
    assert!(response.is_err());
}

#[test_case]
fn test_get_task() {
    let mut client = authorized_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user", "mock_user"]));
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = GetTaskRequest::new(task_id.clone());
    let response = client.get_task(request);
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id);
    let response = unauthorized_client().get_task(request);
    assert!(response.is_err());
}

#[test_case]
fn test_assign_data() {
    let mut client = authorized_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));

    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output" => output_id.clone()),
    );
    let response = unauthorized_client().assign_data(request);
    assert!(response.is_err());

    let request = AssignDataRequest::new(task_id, hashmap!(), hashmap!("output" => output_id));
    let response = client.assign_data(request);
    assert!(response.is_ok());
}

#[test_case]
fn test_approve_task() {
    let mut client = authorized_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    let response = unauthorized_client().approve_task(request);
    assert!(response.is_err());

    let request = ApproveTaskRequest::new(task_id);
    let response = client.approve_task(request);
    assert!(response.is_ok());
}

#[test_case]
fn test_invoke_task() {
    let mut client = authorized_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).unwrap();

    let request = InvokeTaskRequest::new(task_id.clone());
    let response = unauthorized_client().invoke_task(request);
    assert!(response.is_err());

    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request);
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::Staged);

    let mut scheduler_client = get_scheduler_client();
    let executor_id = Uuid::new_v4();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let pull_task_request = PullTaskRequest { executor_id };
    let response = scheduler_client.pull_task(pull_task_request);
    assert!(response.is_ok());
}

#[test_case]
fn test_cancel_task() {
    let mut client = authorized_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).unwrap();

    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request);
    assert!(response.is_ok());

    let mut scheduler_client = get_scheduler_client();

    std::thread::sleep(std::time::Duration::from_secs(5));

    let executor_id = Uuid::new_v4();
    let request = HeartbeatRequest {
        executor_id,
        status: ExecutorStatus::Idle,
    };

    let response = scheduler_client.heartbeat(request).unwrap();
    assert!(response.command == ExecutorCommand::NewTask);

    let request = CancelTaskRequest::new(task_id.clone());
    let response = client.cancel_task(request);
    assert!(response.is_ok());

    std::thread::sleep(std::time::Duration::from_secs(3));

    let pull_task_request = PullTaskRequest { executor_id };
    let response = scheduler_client.pull_task(pull_task_request);
    log::debug!("response: {:?}", response);

    assert!(response.is_err());

    std::thread::sleep(std::time::Duration::from_secs(3));

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).unwrap();

    assert_eq!(response.status, TaskStatus::Canceled);
}

#[test_case]
fn test_fail_task() {
    let mut client = authorized_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).unwrap();

    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request);
    assert!(response.is_ok());

    let mut scheduler_client = get_scheduler_client();

    std::thread::sleep(std::time::Duration::from_secs(5));

    let executor_id = Uuid::new_v4();
    let request = HeartbeatRequest {
        executor_id,
        status: ExecutorStatus::Idle,
    };
    let response = scheduler_client.heartbeat(request).unwrap();
    assert!(response.command == ExecutorCommand::NewTask);

    let pull_task_request = PullTaskRequest { executor_id };
    let response = scheduler_client.pull_task(pull_task_request).unwrap();
    log::debug!("response: {:?}", response);

    let request = HeartbeatRequest {
        executor_id,
        status: ExecutorStatus::Executing,
    };
    let response = scheduler_client.heartbeat(request).unwrap();
    log::debug!("response: {:?}", response);
    assert!(response.command == ExecutorCommand::NoAction);

    std::thread::sleep(std::time::Duration::from_secs(33));

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).unwrap();

    assert_eq!(response.status, TaskStatus::Failed);
}
