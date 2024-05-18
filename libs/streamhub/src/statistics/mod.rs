use {
    super::stream::StreamIdentifier,
    crate::{define::SubscribeType, utils::Uuid},
    chrono::{DateTime, Local},
    serde::Serialize,
    std::{collections::HashMap, sync::Arc, time::Duration},
    tokio::{
        sync::{broadcast::Receiver, Mutex},
        time,
    },
    xflv::define::{AacProfile, AvcCodecId, AvcLevel, AvcProfile, SoundFormat},
};

#[derive(Debug, Clone, Serialize, Default)]
pub struct VideoInfo {
    pub codec: AvcCodecId,
    pub profile: AvcProfile,
    pub level: AvcLevel,
    pub width: u32,
    pub height: u32,
    /*used for caculate the bitrate*/
    #[serde(skip_serializing)]
    pub recv_bytes: u64,
    #[serde(rename = "bitrate(kbits/s)")]
    pub bitrate: u64,
    /*used for caculate the frame rate*/
    #[serde(skip_serializing)]
    pub recv_frame_count: u64,
    pub frame_rate: u64,
    /*used for caculate the GOP*/
    #[serde(skip_serializing)]
    pub recv_frame_count_for_gop: u64,
    pub gop: u64,
}
#[derive(Debug, Clone, Serialize, Default)]
pub struct AudioInfo {
    pub sound_format: SoundFormat,
    pub profile: AacProfile,
    pub samplerate: u32,
    pub channels: u8,
    /*used for caculate the bitrate*/
    #[serde(skip_serializing)]
    pub recv_bytes: u64,
    #[serde(rename = "bitrate(kbits/s)")]
    pub bitrate: u64,
}
#[derive(Debug, Clone, Serialize, Default)]
pub struct StatisticsStream {
    /*publisher infomation */
    pub publisher: StatisticPublisher,
    /*subscriber infomation */
    pub subscribers: HashMap<Uuid, StatisticSubscriber>,
    /*How many clients are subscribing to this stream.*/
    pub subscriber_count: u64,
    /*calculate upstream traffic, now equals audio and video traffic received by this server*/
    pub total_recv_bytes: u64,
    /*calculate downstream traffic, now equals audio and video traffic sent to all subscribers*/
    pub total_send_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Default)]
pub struct StatisticPublisher {
    pub id: Uuid,
    identifier: StreamIdentifier,
    pub start_time: DateTime<Local>,
    pub video: VideoInfo,
    pub audio: AudioInfo,
    pub remote_address: String,
    /*used for caculate the recv_bitrate*/
    #[serde(skip_serializing)]
    pub recv_bytes: u64,
    /*the bitrate at which the server receives streaming data*/
    #[serde(rename = "recv_bitrate(kbits/s)")]
    pub recv_bitrate: u64,
}

impl StatisticPublisher {
    pub fn new(identifier: StreamIdentifier) -> Self {
        Self {
            identifier,
            ..Default::default()
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct StatisticSubscriber {
    pub id: Uuid,
    pub start_time: DateTime<Local>,
    pub remote_address: String,
    pub sub_type: SubscribeType,
    /*used for caculate the send_bitrate*/
    #[serde(skip_serializing)]
    pub send_bytes: u64,
    /*the bitrate at which the server send streaming data to a client*/
    #[serde(rename = "send_bitrate(kbits/s)")]
    pub send_bitrate: u64,
    #[serde(rename = "total_send_bytes(kbits/s)")]
    pub total_send_bytes: u64,
}

impl StatisticsStream {
    pub fn new(identifier: StreamIdentifier) -> Self {
        Self {
            publisher: StatisticPublisher::new(identifier),
            ..Default::default()
        }
    }

    fn get_publisher(&self) -> StatisticsStream {
        let mut statistic_stream = self.clone();
        statistic_stream.subscribers.clear();
        statistic_stream
    }

    fn get_subscriber(&self, uuid: Uuid) -> StatisticsStream {
        let mut statistic_stream = self.clone();
        statistic_stream.subscribers.retain(|&id, _| uuid == id);
        statistic_stream
    }

    pub fn query_by_uuid(&self, uuid: Uuid) -> StatisticsStream {
        if uuid == self.publisher.id {
            self.get_publisher()
        } else {
            self.get_subscriber(uuid)
        }
    }
}

pub struct StatisticsCaculate {
    stream: Arc<Mutex<StatisticsStream>>,
    exit: Receiver<()>,
}

impl StatisticsCaculate {
    pub fn new(stream: Arc<Mutex<StatisticsStream>>, exit: Receiver<()>) -> Self {
        Self { stream, exit }
    }

    async fn caculate(&mut self, seconds: u64) {
        let _start = time::Instant::now();
        let stream_statistics_clone = &mut self.stream.lock().await;

        stream_statistics_clone.publisher.video.bitrate =
            stream_statistics_clone.publisher.video.recv_bytes * 8 / seconds / 1000;
        stream_statistics_clone.publisher.video.recv_bytes = 0;

        stream_statistics_clone.publisher.video.frame_rate =
            stream_statistics_clone.publisher.video.recv_frame_count / seconds;
        stream_statistics_clone.publisher.video.recv_frame_count = 0;

        stream_statistics_clone.publisher.audio.bitrate =
            stream_statistics_clone.publisher.audio.recv_bytes * 8 / seconds / 1000;
        stream_statistics_clone.publisher.audio.recv_bytes = 0;

        stream_statistics_clone.publisher.recv_bitrate =
            stream_statistics_clone.publisher.recv_bytes * 8 / seconds / 1000;
        stream_statistics_clone.publisher.recv_bytes = 0;

        for (_, subscriber) in stream_statistics_clone.subscribers.iter_mut() {
            subscriber.send_bitrate = subscriber.send_bytes * 8 / seconds / 1000;
            subscriber.send_bytes = 0;
        }
        log::info!("caculate statistics cost {:?}", _start.elapsed());
    }
    pub async fn start(&mut self) {
        const INTERVAL: u64 = 10;
        let mut interval = time::interval(Duration::from_secs(INTERVAL));

        loop {
            tokio::select! {
               _ = interval.tick() => {
                self.caculate(INTERVAL).await;
               },
               _ = self.exit.recv() => {
                    log::info!("avstatistics shutting down");
                    return
               },
            }
        }
    }
}
