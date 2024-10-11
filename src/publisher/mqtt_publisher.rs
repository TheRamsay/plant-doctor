use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};

trait Publisher {
    fn publish(&mut self, payload: String);
}

pub struct MqttPublisher<'a> {
    client: EspMqttClient<'a>,
    topic: String,
}

impl<'a> MqttPublisher<'a> {
    pub fn new(client: EspMqttClient<'a>, topic: String) -> Self {
        Self { client, topic }
    }
}

impl<'a> Publisher for MqttPublisher<'a> {
    fn publish(&mut self, payload: String) {
        self.client
            .publish(&self.topic, QoS::AtMostOnce, false, payload.as_bytes())
            .unwrap();
    }
}
