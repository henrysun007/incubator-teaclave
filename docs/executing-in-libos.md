---
permalink: /docs/executing-in-libos
---

# Executing builtin-functions in Occlum

The example shows how to run teaclave builtin-functions in Occlum.

## Requirements
Firstly, get the official occlum Docker image to build teaclave.

```
docker run -it --name teaclave_libos --device /dev/sgx occlum/occlum:0.29.4-ubuntu20.04 
```

Install teaclave's reqirements in contianer. 

```
apt-get update && apt-get install pypy-dev pypy
rustup toolchain install nightly-2022-10-22-x86_64-unknown-linux-gnu
rustup component add rust-src --toolchain nightly-2022-10-22-x86_64-unknown-linux-gnu
```

## Build 

1. Clone the teaclave project.

```
git clone https://github.com/mesatee/incubator-teaclave.git ./teaclave && cd ./teaclave
```

2. Edit the config/build.config.toml and add the executable binary as accepted inbound service of scheduler. The following is an example that uses teaclave_execution_service_libos as the name of binary. 

```
scheduler      = ["teaclave_execution_service", "teaclave_execution_service_libos"]
```

Note that the same name should be used in the build.config.toml and enclave_info.toml.

3. Build teaclave project. After building the project, you can find the binary teaclave_execution_service_libos in ${TEACLAVE_BIN_INSTALL_DIR}. Work at teaclave project source directory.

```
mkdir build && cd build
cmake ..
make
```

4. Build occlum instance. Cmake/scripts/build_instance.sh is a demo script to build an instance.

Note that you should edit the attestation information and the advertised_address of scheduler in runtime.config.toml required by teaclave_execution_service_libos before building the occlum instance.


5. Update enclave_info and auditors for Teaclave platform.

```
sgx_sign dump -enclave ${TEACLAVE_BIN_INSTALL_DIR}/teaclave_instance/build/lib/libocclum-libos.signed.so -dumpfile ${TEACLAVE_OUT_DIR}/teaclave_execution_service_libos_enclave.meta.txt
cat ${TEACLAVE_OUT_DIR}/teaclave_execution_service_libos_enclave.meta.txt | python ${MT_SCRIPT_DIR}/gen_enclave_info_toml.py teaclave_execution_service_libos > ${TEACLAVE_OUT_DIR}/teaclave_execution_service_libos_enclave_info.toml
cd ${TEACLAVE_BUILD_ROOT} && make update_sig
```

## Run

Run teaclave services except teaclave_execution_serice and run teaclave_execution_service_libos on Occlum

```
# Required by teaclave services
mkdir -p /tmp/fusiont_data 
cd ${TEACLAVE_SERVICE_INSTALL_DIR} 

# Before running services, you should check the information in runtime.config.toml.
# For DCAP mode, start the teaclave_dcap_ref_as service first.
./teaclave_authentication_service &
./teaclave_storage_service &
./teaclave_management_service &
./teaclave_scheduler_service &
./teaclave_access_control_service &
./teaclave_frontend_service &

cd $TEACLAVE_BIN_INSTALL_DIR/teaclave_instance && occlum run /bin/teaclave_execution_service_libos

```