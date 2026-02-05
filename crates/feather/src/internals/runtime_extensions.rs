use feather_runtime::http::Response;

/// The **Finalizer API** allows you to send data and terminate
/// the middleware chain in a single, expressive call.
pub trait Finalizer {
    /// Instantly send the Text by returning `end!`
    fn finish_text(&mut self, text: impl Into<String>) -> crate::Outcome;
    /// Instantly send the Html by returning `end!`
    fn finish_html(&mut self, data: impl Into<String>) -> crate::Outcome;
    /// Instantly send the Bytes by returning `end!`
    fn finish_bytes(&mut self, data: impl Into<Vec<u8>>) -> crate::Outcome;
    #[cfg(feature = "json")]
    /// Instantly send the JSON by returning `end!`
    fn finish_json<T: serde::Serialize>(&mut self, data: &T) -> crate::Outcome;
}

impl Finalizer for Response {
    fn finish_text(&mut self, text: impl Into<String>) -> crate::Outcome {
        self.send_text(text);
        Ok(crate::middlewares::MiddlewareResult::End)
    }

    fn finish_html(&mut self, data: impl Into<String>) -> crate::Outcome {
        self.send_html(data);
        Ok(crate::middlewares::MiddlewareResult::End)
    }

    fn finish_bytes(&mut self, data: impl Into<Vec<u8>>) -> crate::Outcome {
        self.send_bytes(data);
        Ok(crate::middlewares::MiddlewareResult::End)
    }

    #[cfg(feature = "json")]
    fn finish_json<T: serde::Serialize>(&mut self, data: &T) -> crate::Outcome {
        self.send_json(data);
        Ok(crate::middlewares::MiddlewareResult::End)
    }
}
