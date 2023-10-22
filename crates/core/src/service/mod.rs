mod backend;
mod error;
pub(crate) mod filter;
mod frontend;

// TODO: place behind feature flag for usage in other crates
#[cfg(test)]
pub mod driver {
    use bytes::BytesMut;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
    use tokio_util::codec::Decoder;

    use crate::messages::{codec::LanguageServerCodec, groups::AllMessages, payload::Payload};

    use super::{
        backend::ServiceBackend,
        error::OUTPUT_CLOSED,
        filter::{IncomingMessage, MessageFilter, OutgoingMessage, ServiceMessageFilter},
        frontend::ServiceFrontend,
    };

    pub struct ServiceDriver<F: MessageFilter> {
        frontend: ServiceFrontend<DuplexStream, DuplexStream>,
        message_filter: ServiceMessageFilter<F>,
        backend: ServiceBackend<F>,
        input_handle: DuplexStream,
        output_handle: DuplexStream,
    }

    impl<F: MessageFilter> ServiceDriver<F> {
        const MAX_PAYLOAD_BYTES: usize = 1_000_000;

        pub async fn send_input_message(&mut self, message: &AllMessages) {
            let payload_string = Payload::new(message).to_string();
            self.send_raw_payload_str(&payload_string).await
        }

        pub async fn send_raw_payload_str(&mut self, payload_str: &str) {
            let bytes_written = self
                .input_handle
                .write(payload_str.as_bytes())
                .await
                .unwrap();

            let payload_size = payload_str.len();

            if bytes_written < payload_size {
                panic!(
                    "unable to send payload in one write, payload size: {}, bytes_written: {}",
                    payload_size, bytes_written
                )
            }
        }

        pub async fn get_output_message(&mut self) -> Option<AllMessages> {
            let mut buffer = vec![0u8; Self::MAX_PAYLOAD_BYTES];
            let bytes_read = self
                .output_handle
                .read(&mut buffer)
                .await
                .expect(OUTPUT_CLOSED);

            LanguageServerCodec::default()
                .decode(&mut BytesMut::from(&buffer[..bytes_read]))
                .expect("invalid payload encoding")
        }

        pub fn get_incoming_at_backend(&mut self) -> Option<IncomingMessage<F>> {
            self.backend.get_incoming()
        }

        pub fn send_outgoing_at_backend(&mut self, message: OutgoingMessage<F>) {
            self.backend.send_outgoing(message)
        }

        pub async fn tick(&mut self) {
            // Backend to frontend
            self.message_filter.tick();
            // Frontend to filter
            self.frontend.tick().await;
            // Filter to backend
            self.message_filter.tick();
        }
    }

    impl<F: MessageFilter> Default for ServiceDriver<F> {
        fn default() -> Self {
            let (frontend_tx, frontend_rx) = futures::channel::mpsc::unbounded::<AllMessages>();
            let (message_filter_tx, message_filter_rx) =
                futures::channel::mpsc::unbounded::<AllMessages>();
            let (incoming_tx, incoming_rx) =
                futures::channel::mpsc::unbounded::<IncomingMessage<F>>();
            let (outgoing_tx, outgoing_rx) =
                futures::channel::mpsc::unbounded::<OutgoingMessage<F>>();
            let (service_input, input_handle) = tokio::io::duplex(Self::MAX_PAYLOAD_BYTES);
            let (service_output, output_handle) = tokio::io::duplex(Self::MAX_PAYLOAD_BYTES);

            Self {
                frontend: ServiceFrontend::new(
                    service_input,
                    service_output,
                    frontend_tx,
                    message_filter_rx,
                ),
                message_filter: ServiceMessageFilter::new(
                    frontend_rx,
                    message_filter_tx,
                    outgoing_rx,
                    incoming_tx,
                ),
                backend: ServiceBackend::new(incoming_rx, outgoing_tx),
                input_handle,
                output_handle,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::{
            core::response::{
                response_error::{ReservedResponseErrorCodes, ResponseErrorCode},
                ResponseMessage, UntypedResponseMessage,
            },
            groups::{responses::errors::ErrorResponse, tests::MESSAGE_MOCK, AllMessages},
            payload::tests::INVALID_PAYLOAD_STR_MOCK,
        },
        service::{
            driver::ServiceDriver,
            filter::tests::{
                FilterMock, INCOMING_MESSAGE_MOCK, INVALID_INCOMING_MOCK, OUTGOING_MESSAGE_MOCK,
            },
        },
    };

    fn assert_response_message(message: AllMessages) -> UntypedResponseMessage {
        match message {
            AllMessages::UntypedResponse(untyped_response) => untyped_response,
            _ => panic!(),
        }
    }

    #[test_log::test(tokio::test)]
    async fn forwards_payload_to_backend() {
        let mut service_driver = ServiceDriver::<FilterMock>::default();
        service_driver.send_input_message(&MESSAGE_MOCK).await;
        service_driver.tick().await;
        assert!(service_driver
            .get_incoming_at_backend()
            .is_some_and(|message| message == INCOMING_MESSAGE_MOCK))
    }

    #[test_log::test(tokio::test)]
    async fn fails_on_invalid_incoming() {
        let mut service_driver = ServiceDriver::<FilterMock>::default();
        service_driver
            .send_input_message(&INVALID_INCOMING_MOCK)
            .await;
        // Send message to message filter
        service_driver.tick().await;
        // Pull error message message filter.
        service_driver.tick().await;
        assert!(service_driver
            .get_output_message()
            .await
            .is_some_and(|message| {
                let response_message = assert_response_message(message);
                ResponseMessage::<ErrorResponse>::try_from(response_message)
                    .unwrap()
                    .kind
                    .is_err_and(|err| err.message.contains("invalid message"))
            }))
    }

    #[test_log::test(tokio::test)]
    async fn outputs_payload_from_backend() {
        let mut service_driver = ServiceDriver::<FilterMock>::default();
        service_driver.send_outgoing_at_backend(OUTGOING_MESSAGE_MOCK);
        service_driver.tick().await;
        assert!(service_driver
            .get_output_message()
            .await
            .is_some_and(|message| message == MESSAGE_MOCK))
    }

    #[test_log::test(tokio::test)]
    async fn responds_with_decode_error_for_invalid_payload() {
        let mut service_driver = ServiceDriver::<FilterMock>::default();
        service_driver
            .send_raw_payload_str(&INVALID_PAYLOAD_STR_MOCK)
            .await;
        service_driver.tick().await;
        assert!(service_driver
            .get_output_message()
            .await
            .is_some_and(|message| {
                let response_message = assert_response_message(message);
                ResponseMessage::<ErrorResponse>::try_from(response_message)
                    .unwrap()
                    .kind
                    .is_err_and(|err| {
                        err.code
                            == ResponseErrorCode::Reserved(ReservedResponseErrorCodes::ParseError)
                    })
            }))
    }
}
