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
use teaclave_proto::teaclave_access_control_service::*;
use teaclave_test_utils::test_case;

#[test_case]
fn test_authorize_data_success() {
    let mut client = get_access_control_client();

    let request = AuthorizeDataRequest::new("mock_user_a", "mock_data");
    let response_result = client.authorize_data(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

#[test_case]
fn test_authorize_data_fail() {
    let mut client = get_access_control_client();

    let request = AuthorizeDataRequest::new("mock_user_d", "mock_data");
    let response_result = client.authorize_data(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);

    let request = AuthorizeDataRequest::new("mock_user_a", "mock_data_b");
    let response_result = client.authorize_data(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

#[test_case]
fn test_authorize_function_success() {
    let mut client = get_access_control_client();

    let request =
        AuthorizeFunctionRequest::new("mock_public_function_owner", "mock_public_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);

    let request =
        AuthorizeFunctionRequest::new("mock_private_function_owner", "mock_private_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);

    let request =
        AuthorizeFunctionRequest::new("mock_private_function_owner", "mock_public_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

#[test_case]
fn test_authorize_function_fail() {
    let mut client = get_access_control_client();
    let request =
        AuthorizeFunctionRequest::new("mock_public_function_owner", "mock_private_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

#[test_case]
fn test_authorize_task_success() {
    let mut client = get_access_control_client();
    let request = AuthorizeTaskRequest::new("mock_participant_a", "mock_task");
    let response_result = client.authorize_task(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);

    let request = AuthorizeTaskRequest::new("mock_participant_b", "mock_task");
    let response_result = client.authorize_task(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

#[test_case]
fn test_authorize_task_fail() {
    let mut client = get_access_control_client();
    let request = AuthorizeTaskRequest::new("mock_participant_c", "mock_task");
    let response_result = client.authorize_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

#[test_case]
fn test_authorize_staged_task_success() {
    let mut client = get_access_control_client();
    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_allowed_private_function".to_string(),
        object_input_data_id_list: vec![
            "mock_staged_allowed_data1".to_string(),
            "mock_staged_allowed_data2".to_string(),
            "mock_staged_allowed_data3".to_string(),
        ],
        object_output_data_id_list: vec![
            "mock_staged_allowed_data1".to_string(),
            "mock_staged_allowed_data2".to_string(),
            "mock_staged_allowed_data3".to_string(),
        ],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

#[test_case]
fn test_authorize_staged_task_fail() {
    let mut client = get_access_control_client();
    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_disallowed_private_function".to_string(),
        object_input_data_id_list: vec![],
        object_output_data_id_list: vec![],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);

    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_allowed_private_function".to_string(),
        object_input_data_id_list: vec!["mock_staged_disallowed_data1".to_string()],
        object_output_data_id_list: vec![],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);

    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_allowed_private_function".to_string(),
        object_input_data_id_list: vec![],
        object_output_data_id_list: vec!["mock_staged_disallowed_data2".to_string()],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

#[test_case]
fn test_concurrency() {
    let mut thread_pool = Vec::new();
    for _i in 0..10 {
        let child = std::thread::spawn(move || {
            for _j in 0..10 {
                test_authorize_data_fail();
                test_authorize_function_fail();
                test_authorize_task_success();
                test_authorize_staged_task_fail();
            }
        });
        thread_pool.push(child);
    }
    for thr in thread_pool.into_iter() {
        assert!(thr.join().is_ok());
    }
}
