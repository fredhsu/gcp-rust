# gcp-rust
Building this as an exercise in Rust and APIs

TODO:
* add webserver
* add threads
* use enums when appropriate
* remove static stuff
* add tests
* provide helper operations
        ```
        let rsp = client
        .<operation_name>().
        .<param>("some value")
        .send().await;
        ```
* create client for each resource type
* create builders
    ```
    let config = aws_sdk_ec2::config::Builder::from(&shared_config)
  .retry_config(RetryConfig::disabled())
  .build();
    ```
* code for oauth/token retrieval
