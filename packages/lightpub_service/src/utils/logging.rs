pub fn report_error<R, E: std::fmt::Debug>(result: Result<R, E>, msg_prefix: &str) -> Result<R, E> {
    if let Err(e) = &result {
        tracing::error!("{msg_prefix}: {:#?}", e);
    }
    result
}
