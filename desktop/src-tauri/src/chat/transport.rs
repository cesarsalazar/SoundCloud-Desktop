use chat_core::{
    Tier, Transport, TransportError, TransportRequest, TransportResponse, async_trait,
};

pub struct EdgeTransport;

#[async_trait]
impl Transport for EdgeTransport {
    fn tiers(&self) -> Vec<Tier> {
        vec![Tier::Direct, Tier::Relay, Tier::Pro]
    }

    async fn exchange(
        &self,
        _request: TransportRequest,
    ) -> Result<TransportResponse, TransportError> {
        Err(TransportError::Unreachable)
    }
}
