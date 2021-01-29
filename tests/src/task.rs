// Copyright 2015-2020 Aerospike, Inc.
//
// Portions may be licensed to Aerospike, Inc. under one or more contributor
// license agreements.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use crate::common;
use aerospike::errors::ErrorKind;
use aerospike::task::{Status, Task};
use aerospike::*;
use std::thread;
use std::time::Duration;

// If registering udf is successful, querying RegisterTask will return Status::Complete
// If udf does not exist, querying RegisterTask will return error
#[test]
fn register_task_test() {
    let client = common::client();

    let code = r#"
    local function putBin(r,name,value)
        if not aerospike:exists(r) then aerospike:create(r) end
        r[name] = value
        aerospike:update(r)
    end
    function writeBin(r,name,value)
        putBin(r,name,value)
    end
    "#;

    let udf_name = common::rand_str(10);
    let udf_file_name = udf_name.to_owned() + ".LUA";

    let register_task = client
        .register_udf(
            &WritePolicy::default(),
            code.as_bytes(),
            &udf_file_name,
            UDFLang::Lua,
        )
        .unwrap();

    assert!(matches!(
        register_task.wait_till_complete(None),
        Ok(Status::Complete)
    ));

    client
        .remove_udf(&WritePolicy::default(), &udf_name, UDFLang::Lua)
        .unwrap();
    // Wait for some time to ensure UDF has been unregistered on all nodes.
    thread::sleep(Duration::from_secs(2));

    let timeout = Duration::from_millis(100);
    assert!(matches!(
        register_task.wait_till_complete(Some(timeout)),
        Err(Error(ErrorKind::Timeout(_), _))
    ));
}

// If creating index is successful, querying IndexTask will return Status::Complete
#[test]
fn index_task_test() {
    let client = common::client();
    let namespace = common::namespace();
    let set_name = common::rand_str(10);
    let bin_name = common::rand_str(10);
    let index_name = common::rand_str(10);

    let wpolicy = WritePolicy::default();
    for i in 0..2_i64 {
        let key = as_key!(namespace, &set_name, i);
        let wbin = as_bin!(&bin_name, i);
        let bins = vec![&wbin];
        client.put(&wpolicy, &key, &bins).unwrap();
    }

    let index_task = client
        .create_index(
            &wpolicy,
            &namespace,
            &set_name,
            &bin_name,
            &index_name,
            IndexType::Numeric,
        )
        .unwrap();

    assert!(matches!(
        index_task.wait_till_complete(None),
        Ok(Status::Complete)
    ));
}
