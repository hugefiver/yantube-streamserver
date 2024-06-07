use webrtc::api::media_engine::MediaEngine;
use webrtc::error::Result;
use webrtc::interceptor::registry::Registry;
use webrtc::interceptor::twcc::receiver::ReceiverBuilder;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpHeaderExtensionCapability, RTPCodecType};
use webrtc::rtp_transceiver::{RTCPFeedback, TYPE_RTCP_FB_TRANSPORT_CC};

pub use webrtc::api::interceptor_registry::{
    self as webrtc_interceptors_registry, configure_nack, configure_rtcp_reports,
};
use webrtc::sdp;

pub fn apply_default_interceptors(
    mut registry: Registry, media_engine: &mut MediaEngine,
) -> Result<Registry> {
    registry = configure_nack(registry, media_engine);

    registry = configure_rtcp_reports(registry);

    // registry = configure_twcc_receiver_only(registry, media_engine)?;

    Ok(registry)
}

// configure_twcc_receiver will setup everything necessary for generating TWCC reports.
pub fn configure_twcc_receiver_only_with_receiver(
    mut registry: Registry, media_engine: &mut MediaEngine, receiver: ReceiverBuilder,
) -> Result<Registry> {
    media_engine.register_feedback(
        RTCPFeedback {
            typ: TYPE_RTCP_FB_TRANSPORT_CC.to_owned(),
            ..Default::default()
        },
        RTPCodecType::Video,
    );
    media_engine.register_header_extension(
        RTCRtpHeaderExtensionCapability {
            uri: sdp::extmap::TRANSPORT_CC_URI.to_owned(),
        },
        RTPCodecType::Video,
        None,
    )?;

    media_engine.register_feedback(
        RTCPFeedback {
            typ: TYPE_RTCP_FB_TRANSPORT_CC.to_owned(),
            ..Default::default()
        },
        RTPCodecType::Audio,
    );
    media_engine.register_header_extension(
        RTCRtpHeaderExtensionCapability {
            uri: sdp::extmap::TRANSPORT_CC_URI.to_owned(),
        },
        RTPCodecType::Audio,
        None,
    )?;

    let receiver = Box::new(receiver);
    registry.add(receiver);
    Ok(registry)
}
