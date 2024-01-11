<div align="center">
  <h1>
    <code>tcp_env_logger</code>
  </h1>
  <strong>TCP logger built on top of env_logger</strong>
</div>

## Example usage
Note that `log_hostname` can be any arbitrary value. Its purpose is to identify
the server emitting the logs.

`log_url` is the full TCP socket connection URL.

```rs
fn setup_logger() {
    let log_hostname = std::env::var("LOG_LOCAL_HOSTNAME").expect("LOG_LOCAL_HOSTNAME not defined");
    let log_url = std::env::var("LOG_REMOTE_URL").expect("LOG_REMOTE_URL not defined");
    let env_logger = env_logger::Builder::from_default_env().build();
    EnvTcpLogger::init(log_hostname, log_url, env_logger).unwrap();
}
```
