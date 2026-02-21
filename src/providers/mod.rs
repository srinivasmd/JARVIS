#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub user_input: String,
    pub context: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub output: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    Unavailable,
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Unavailable => write!(f, "provider unavailable"),
        }
    }
}

impl std::error::Error for ProviderError {}

pub trait LlmProvider {
    fn name(&self) -> &str;
    fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError>;
}

pub struct EchoProvider;

impl LlmProvider for EchoProvider {
    fn name(&self) -> &str {
        "echo-local"
    }

    fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let prefix = if request.context.is_empty() {
            ""
        } else {
            "[ctx] "
        };
        Ok(ProviderResponse {
            output: format!("{}{}", prefix, request.user_input),
            model: self.name().to_string(),
        })
    }
}

pub struct ProviderRouter {
    providers: Vec<Box<dyn LlmProvider>>,
}

impl ProviderRouter {
    pub fn new(providers: Vec<Box<dyn LlmProvider>>) -> Self {
        Self { providers }
    }

    pub fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        for provider in &self.providers {
            if let Ok(response) = provider.generate(request) {
                return Ok(response);
            }
        }
        Err(ProviderError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingProvider;

    impl LlmProvider for FailingProvider {
        fn name(&self) -> &str {
            "down"
        }

        fn generate(&self, _request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
            Err(ProviderError::Unavailable)
        }
    }

    #[test]
    fn router_falls_back_to_next_provider() {
        let router = ProviderRouter::new(vec![Box::new(FailingProvider), Box::new(EchoProvider)]);
        let response = router
            .generate(&ProviderRequest {
                user_input: "hello".to_string(),
                context: vec![],
            })
            .expect("fallback provider should respond");

        assert_eq!(response.output, "hello");
        assert_eq!(response.model, "echo-local");
    }
}
