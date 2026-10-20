use crate::channel::{Channel, InnerCall, Sealed};
use crate::contracts::{ChatError, ChatResult, DeviceId, Tier};
use crate::crypto::DeviceKeys;

pub fn seal_login(
    _channel: &Channel,
    _keys: &DeviceKeys,
    _device_id: DeviceId,
    _call: &InnerCall,
    _target_session: Option<&str>,
    _tier: Tier,
) -> ChatResult<Sealed> {
    Err(ChatError::Unimplemented)
}
