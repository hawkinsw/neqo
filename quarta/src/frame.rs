// TODO(ekr@rtfm.com): Remove this once I've implemented everything.
#![allow(unused_variables, dead_code)]
use super::data::*;
use super::*;

const FRAME_TYPE_PADDING: u8 = 0x0;
const FRAME_TYPE_PING: u8 = 0x1;
const FRAME_TYPE_ACK: u8 = 0x2;
const FRAME_TYPE_ACK_ECN: u8 = 0x3;
const FRAME_TYPE_RST_STREAM: u8 = 0x4;
const FRAME_TYPE_STOP_SENDING: u8 = 0x5;
const FRAME_TYPE_CRYPTO: u8 = 0x6;
const FRAME_TYPE_NEW_TOKEN: u8 = 0x7;
const FRAME_TYPE_STREAM: u8 = 0x8;
const FRAME_TYPE_STREAM_MAX: u8 = 0xf;
const FRAME_TYPE_MAX_DATA: u8 = 0x10;
const FRAME_TYPE_MAX_STREAM_DATA: u8 = 0x11;
const FRAME_TYPE_MAX_STREAMS_BIDI: u8 = 0x12;
const FRAME_TYPE_MAX_STREAMS_UNIDI: u8 = 0x13;
const FRAME_TYPE_DATA_BLOCKED: u8 = 0x14;
const FRAME_TYPE_STREAM_DATA_BLOCKED: u8 = 0x15;
const FRAME_TYPE_STREAMS_BLOCKED_BIDI: u8 = 0x16;
const FRAME_TYPE_STREAMS_BLOCKED_UNIDI: u8 = 0x17;
const FRAME_TYPE_NEW_CONNECTION_ID: u8 = 0x18;
const FRAME_TYPE_RETIRE_CONNECTION_ID: u8 = 0x19;
const FRAME_TYPE_PATH_CHALLENGE: u8 = 0x1a;
const FRAME_TYPE_PATH_RESPONSE: u8 = 0x1b;
const FRAME_TYPE_CONNECTION_CLOSE_TRANSPORT: u8 = 0x1c;
const FRAME_TYPE_CONNECTION_CLOSE_APPLICATION: u8 = 0x1d;

const STREAM_FRAME_BIT_FIN: u8 = 0x01;
const STREAM_FRAME_BIT_LEN: u8 = 0x02;
const STREAM_FRAME_BIT_OFF: u8 = 0x04;

#[derive(PartialEq, Debug)]
enum StreamType {
    BiDi,
    UniDi,
}

impl StreamType {
    fn frame_type_bit(&self) -> u8 {
        match self {
            StreamType::BiDi => 0,
            StreamType::UniDi => 1,
        }
    }
    fn from_type_bit(bit: u8) -> StreamType {
        if (bit & 0x01) == 0 {
            return StreamType::BiDi;
        }
        return StreamType::UniDi;
    }
}

#[derive(PartialEq, Debug)]
enum CloseType {
    Transport,
    Application,
}

impl CloseType {
    fn frame_type_bit(&self) -> u8 {
        match self {
            CloseType::Transport => 0,
            CloseType::Application => 1,
        }
    }
    fn from_type_bit(bit: u8) -> CloseType {
        if (bit & 0x01) == 0 {
            return CloseType::Transport;
        }
        return CloseType::Application;
    }
}

#[derive(PartialEq, Debug, Default)]
struct AckRange {
    gap: u64,
    range: u64,
}

#[derive(PartialEq, Debug)]
enum Frame {
    Padding,
    Ping,
    Ack {
        largest_acknowledged: u64,
        ack_delay: u64,
        first_ack_range: u64,
        ack_ranges: Vec<AckRange>,
    },
    ResetStream {
        stream_id: u64,
        application_error_code: u16,
        final_size: u64,
    },
    StopSending {
        application_error_code: u16,
    },
    Crypto {
        offset: u64,
        data: Vec<u8>,
    },
    NewToken {
        token: Vec<u8>,
    },
    Stream {
        fin: bool,
        stream_id: u64,
        offset: u64,
        data: Vec<u8>,
    },
    MaxData {
        maximum_data: u64,
    },
    MaxStreamData {
        stream_id: u64,
        maximum_stream_data: u64,
    },
    MaxStreams {
        stream_type: StreamType,
        maximum_streams: u64,
    },
    DataBlocked {
        data_limit: u64,
    },
    StreamDataBlocked {
        stream_id: u64,
        stream_data_limit: u64,
    },
    StreamsBlocked {
        stream_type: StreamType,
        stream_limit: u64,
    },
    NewConnectionId {
        sequence_number: u64,
        connection_id: Vec<u8>,
        stateless_reset_token: [u8; 16],
    },
    RetireConnectionId {
        sequence_number: u64,
    },
    PathChallenge {
        data: [u8; 8],
    },
    PathResponse {
        data: [u8; 8],
    },
    ConnectionClose {
        close_type: CloseType,
        error_code: u16,
        frame_type: u64,
        reason_phrase: Vec<u8>,
    },
}

impl Frame {
    fn get_type(&self) -> u8 {
        match self {
            Frame::Padding => FRAME_TYPE_PADDING,
            Frame::Ping => FRAME_TYPE_PING,
            Frame::Ack { .. } => FRAME_TYPE_ACK, // We don't do ACK ECN.
            Frame::ResetStream { .. } => FRAME_TYPE_RST_STREAM,
            Frame::StopSending { .. } => FRAME_TYPE_STOP_SENDING,
            Frame::Crypto { .. } => FRAME_TYPE_CRYPTO,
            Frame::NewToken { .. } => FRAME_TYPE_NEW_TOKEN,
            Frame::Stream { fin, offset, .. } => {
                let mut t = FRAME_TYPE_STREAM;
                if *fin {
                    t = t | STREAM_FRAME_BIT_FIN;
                }
                if *offset > 0 {
                    t = t | STREAM_FRAME_BIT_OFF;
                }
                t = t | STREAM_FRAME_BIT_LEN;
                t
            }
            Frame::MaxData { .. } => FRAME_TYPE_MAX_DATA,
            Frame::MaxStreamData { .. } => FRAME_TYPE_MAX_STREAM_DATA,
            Frame::MaxStreams { stream_type, .. } => {
                FRAME_TYPE_MAX_STREAMS_BIDI + stream_type.frame_type_bit()
            }
            Frame::DataBlocked { .. } => FRAME_TYPE_DATA_BLOCKED,
            Frame::StreamDataBlocked { .. } => FRAME_TYPE_STREAM_DATA_BLOCKED,
            Frame::StreamsBlocked { stream_type, .. } => {
                FRAME_TYPE_STREAMS_BLOCKED_BIDI + stream_type.frame_type_bit()
            }
            Frame::NewConnectionId { .. } => FRAME_TYPE_NEW_CONNECTION_ID,
            Frame::RetireConnectionId { .. } => FRAME_TYPE_RETIRE_CONNECTION_ID,
            Frame::PathChallenge { .. } => FRAME_TYPE_PATH_CHALLENGE,
            Frame::PathResponse { .. } => FRAME_TYPE_PATH_RESPONSE,
            Frame::ConnectionClose { close_type, .. } => {
                FRAME_TYPE_CONNECTION_CLOSE_TRANSPORT + close_type.frame_type_bit()
            }
        }
    }

    fn marshal(&self, d: &mut Data) -> Res<()> {
        d.encode_byte(self.get_type());

        match self {
            Frame::Padding => (),
            Frame::Ping => (),
            Frame::Ack {
                largest_acknowledged,
                ack_delay,
                first_ack_range,
                ack_ranges,
            } => {
                d.encode_varint(*largest_acknowledged);
                d.encode_varint(*ack_delay);
                d.encode_varint(ack_ranges.len() as u64);
                d.encode_varint(*first_ack_range);
                for ref r in ack_ranges {
                    d.encode_varint(r.gap);
                    d.encode_varint(r.range);
                }
            }
            Frame::ResetStream {
                stream_id,
                application_error_code,
                final_size,
            } => {
                d.encode_varint(*stream_id);
                d.encode_uint(*application_error_code, 2);
                d.encode_varint(*final_size);
            }
            Frame::StopSending {
                application_error_code,
            } => {
                d.encode_uint(*application_error_code, 2);
            }
            Frame::Crypto { offset, data } => {
                d.encode_varint(*offset);
                d.encode_vec_and_len(&data);
            }
            Frame::NewToken { token } => {
                d.encode_vec_and_len(token);
            }
            Frame::Stream {
                stream_id,
                offset,
                data,
                ..
            } => {
                d.encode_varint(*stream_id);
                if *offset > 0 {
                    d.encode_varint(*offset);
                }
                d.encode_vec_and_len(&data);
            }
            Frame::MaxData { maximum_data } => {
                d.encode_varint(*maximum_data);
            }
            Frame::MaxStreamData {
                stream_id,
                maximum_stream_data,
            } => {
                d.encode_varint(*stream_id);
                d.encode_varint(*maximum_stream_data);
            }
            Frame::MaxStreams {
                maximum_streams, ..
            } => {
                d.encode_varint(*maximum_streams);
            }
            Frame::DataBlocked { data_limit } => {
                d.encode_varint(*data_limit);
            }
            Frame::StreamDataBlocked {
                stream_id,
                stream_data_limit,
            } => {
                d.encode_varint(*stream_id);
                d.encode_varint(*stream_data_limit);
            }
            Frame::StreamsBlocked { stream_limit, .. } => {
                d.encode_varint(*stream_limit);
            }
            Frame::NewConnectionId {
                sequence_number,
                connection_id,
                stateless_reset_token,
            } => {
                d.encode_varint(*sequence_number);
                d.encode_uint(connection_id.len() as u64, 1);
                d.encode_vec(connection_id);
                d.encode_vec(stateless_reset_token);
            }
            Frame::RetireConnectionId { sequence_number } => {
                d.encode_varint(*sequence_number);
            }
            Frame::PathChallenge { data } => {
                d.encode_vec(data);
            }
            Frame::PathResponse { data } => {
                d.encode_vec(data);
            }
            Frame::ConnectionClose {
                error_code,
                frame_type,
                reason_phrase,
                ..
            } => {
                d.encode_uint(*error_code, 2);
                d.encode_varint(*frame_type);
                d.encode_vec_and_len(reason_phrase);
            }
        }

        Ok(())
    }
}

fn decode_frame(d: &mut Data) -> Res<Frame> {
    // TODO(ekr@rtfm.com): check for minimal encoding
    let t = d.decode_byte()?;

    match t {
        FRAME_TYPE_PADDING => Ok(Frame::Padding),
        FRAME_TYPE_PING => Ok(Frame::Ping),
        FRAME_TYPE_RST_STREAM => Ok(Frame::ResetStream {
            stream_id: d.decode_varint()?,
            application_error_code: d.decode_uint(2)? as u16,
            final_size: d.decode_varint()?,
        }),
        FRAME_TYPE_ACK | FRAME_TYPE_ACK_ECN => {
            let la = d.decode_varint()?;
            let ad = d.decode_varint()?;
            let nr = d.decode_varint()?;
            let fa = d.decode_varint()?;
            let mut arr: Vec<AckRange> = Vec::with_capacity(nr as usize);
            for i in 0..nr {
                let ar = AckRange {
                    gap: d.decode_varint()?,
                    range: d.decode_varint()?,
                };
                arr.push(ar);
            }

            // Now check for the values for ACK_ECN.
            if t == FRAME_TYPE_ACK_ECN {
                d.decode_varint()?;
                d.decode_varint()?;
                d.decode_varint()?;
            }

            Ok(Frame::Ack {
                largest_acknowledged: la,
                ack_delay: ad,
                first_ack_range: fa,
                ack_ranges: arr,
            })
        }
        FRAME_TYPE_STOP_SENDING => Ok(Frame::StopSending {
            application_error_code: d.decode_uint(2)? as u16,
        }),
        FRAME_TYPE_CRYPTO => {
            let o = d.decode_varint()?;
            let l = d.decode_varint()?;
            Ok(Frame::Crypto {
                offset: o,
                data: d.decode_data(l as usize)?,
            })
        }
        FRAME_TYPE_NEW_TOKEN => {
            let l = d.decode_varint()?;
            Ok(Frame::NewToken {
                token: d.decode_data(l as usize)?,
            })
        }
        FRAME_TYPE_STREAM...FRAME_TYPE_STREAM_MAX => {
            let s = d.decode_varint()?;
            let mut o: u64 = 0;
            if (t & STREAM_FRAME_BIT_OFF) != 0 {
                o = d.decode_varint()?;
            }
            let mut l: u64;
            let mut data: Vec<u8>;
            println!("{}", t);
            if (t & STREAM_FRAME_BIT_LEN) != 0 {
                println!("Decoding length");
                l = d.decode_varint()?;
                data = d.decode_data(l as usize)?;
            } else {
                println!("Decoding without length");
                data = d.decode_remainder()?;
            }
            Ok(Frame::Stream {
                fin: (t & STREAM_FRAME_BIT_FIN) != 0,
                stream_id: s,
                offset: o,
                data: data,
            })
        }
        FRAME_TYPE_MAX_DATA => Ok(Frame::MaxData {
            maximum_data: d.decode_varint()?,
        }),
        FRAME_TYPE_MAX_STREAM_DATA => Ok(Frame::MaxStreamData {
            stream_id: d.decode_varint()?,
            maximum_stream_data: d.decode_varint()?,
        }),
        FRAME_TYPE_MAX_STREAMS_BIDI | FRAME_TYPE_MAX_STREAMS_UNIDI => Ok(Frame::MaxStreams {
            stream_type: StreamType::from_type_bit(t),
            maximum_streams: d.decode_varint()?,
        }),

        FRAME_TYPE_DATA_BLOCKED => Ok(Frame::DataBlocked {
            data_limit: d.decode_varint()?,
        }),
        FRAME_TYPE_STREAM_DATA_BLOCKED => Ok(Frame::StreamDataBlocked {
            stream_id: d.decode_varint()?,
            stream_data_limit: d.decode_varint()?,
        }),
        FRAME_TYPE_STREAMS_BLOCKED_BIDI | FRAME_TYPE_STREAMS_BLOCKED_UNIDI => {
            Ok(Frame::StreamsBlocked {
                stream_type: StreamType::from_type_bit(t),
                stream_limit: d.decode_varint()?,
            })
        }
        FRAME_TYPE_NEW_CONNECTION_ID => {
            let s = d.decode_varint()?;
            let l = d.decode_uint(1)?;
            let cid = d.decode_data(l as usize)?;
            let srt = d.decode_data(16)?;
            let mut srtv: [u8; 16] = [0; 16];
            srtv.copy_from_slice(&srt);

            Ok(Frame::NewConnectionId {
                sequence_number: s,
                connection_id: cid,
                stateless_reset_token: srtv,
            })
        }
        FRAME_TYPE_RETIRE_CONNECTION_ID => Ok(Frame::RetireConnectionId {
            sequence_number: d.decode_varint()?,
        }),
        FRAME_TYPE_PATH_CHALLENGE => {
            let data = d.decode_data(8)?;
            let mut datav: [u8; 8] = [0; 8];
            datav.copy_from_slice(&data);
            Ok(Frame::PathChallenge { data: datav })
        }
        FRAME_TYPE_PATH_RESPONSE => {
            let data = d.decode_data(8)?;
            let mut datav: [u8; 8] = [0; 8];
            datav.copy_from_slice(&data);
            Ok(Frame::PathResponse { data: datav })
        }
        FRAME_TYPE_CONNECTION_CLOSE_TRANSPORT | FRAME_TYPE_CONNECTION_CLOSE_APPLICATION => {
            Ok(Frame::ConnectionClose {
                close_type: CloseType::from_type_bit(t),
                error_code: d.decode_uint(2)? as u16,
                frame_type: d.decode_varint()?,
                reason_phrase: d.decode_data_and_len()?,
            })
        }
        _ => Err(Error::ErrUnknownFrameType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc_dec(f: &Frame, s: &str) {
        let mut d = Data::default();

        f.marshal(&mut d).unwrap();
        assert_eq!(d, Data::from_hex(s));

        let f2 = decode_frame(&mut d).unwrap();
        assert_eq!(*f, f2);
    }

    #[test]
    fn test_padding_frame() {
        let f = Frame::Padding;
        enc_dec(&f, "00");
    }

    #[test]
    fn test_ping_frame() {
        let f = Frame::Ping;
        enc_dec(&f, "01");
    }

    #[test]
    fn test_ack() {
        let ar = vec![AckRange { gap: 1, range: 2 }, AckRange { gap: 3, range: 4 }];

        let f = Frame::Ack {
            largest_acknowledged: 0x1234,
            ack_delay: 0x1235,
            first_ack_range: 0x1236,
            ack_ranges: ar,
        };

        enc_dec(&f, "025234523502523601020304");

        // Try to parse ACK_ECN without ECN values
        let mut d1 = Data::from_hex("035234523502523601020304");
        assert_eq!(decode_frame(&mut d1).unwrap_err(), Error::ErrNoMoreData);

        // Try to parse ACK_ECN without ECN values
        d1 = Data::from_hex("035234523502523601020304010203");
        assert_eq!(decode_frame(&mut d1).unwrap(), f);
    }

    #[test]
    fn test_reset_stream() {
        let f = Frame::ResetStream {
            stream_id: 0x1234,
            application_error_code: 0x77,
            final_size: 0x3456,
        };

        enc_dec(&f, "04523400777456");
    }

    #[test]
    fn test_stop_sending() {
        let f = Frame::StopSending {
            application_error_code: 0x77,
        };

        enc_dec(&f, "050077")
    }

    #[test]
    fn test_crypto() {
        let f = Frame::Crypto {
            offset: 1,
            data: vec![1, 2, 3],
        };

        enc_dec(&f, "060103010203");
    }

    #[test]
    fn test_new_token() {
        let f = Frame::NewToken {
            token: vec![0x12, 0x34, 0x56],
        };

        enc_dec(&f, "0703123456");
    }

    #[test]
    fn test_stream() {
        // First, just set the length bit.
        let mut f = Frame::Stream {
            fin: false,
            stream_id: 5,
            offset: 0,
            data: vec![1, 2, 3],
        };

        enc_dec(&f, "0a0503010203");

        // Now verify that we can parse without the length
        // bit, because we never generate this.
        let mut d1 = Data::from_hex("0805010203");
        let f2 = decode_frame(&mut d1).unwrap();
        assert_eq!(f, f2);

        // Now with offset != 0 and FIN
        f = Frame::Stream {
            fin: true,
            stream_id: 5,
            offset: 1,
            data: vec![1, 2, 3],
        };
        enc_dec(&f, "0f050103010203");
    }

    #[test]
    fn test_max_data() {
        let f = Frame::MaxData {
            maximum_data: 0x1234,
        };

        enc_dec(&f, "105234");
    }

    #[test]
    fn test_max_stream_data() {
        let f = Frame::MaxStreamData {
            stream_id: 5,
            maximum_stream_data: 0x1234,
        };

        enc_dec(&f, "11055234");
    }

    #[test]
    fn test_max_streams() {
        let mut f = Frame::MaxStreams {
            stream_type: StreamType::BiDi,
            maximum_streams: 0x1234,
        };

        enc_dec(&f, "125234");

        f = Frame::MaxStreams {
            stream_type: StreamType::UniDi,
            maximum_streams: 0x1234,
        };

        enc_dec(&f, "135234");
    }

    #[test]
    fn test_data_blocked() {
        let f = Frame::DataBlocked { data_limit: 0x1234 };

        enc_dec(&f, "145234");
    }

    #[test]
    fn test_stream_data_blocked() {
        let f = Frame::StreamDataBlocked {
            stream_id: 5,
            stream_data_limit: 0x1234,
        };

        enc_dec(&f, "15055234");
    }

    #[test]
    fn test_streams_blocked() {
        let mut f = Frame::StreamsBlocked {
            stream_type: StreamType::BiDi,
            stream_limit: 0x1234,
        };

        enc_dec(&f, "165234");

        f = Frame::StreamsBlocked {
            stream_type: StreamType::UniDi,
            stream_limit: 0x1234,
        };

        enc_dec(&f, "175234");
    }

    #[test]
    fn test_new_connection_id() {
        let f = Frame::NewConnectionId {
            sequence_number: 0x1234,
            connection_id: vec![0x01, 0x02],
            stateless_reset_token: [9; 16],
        };

        enc_dec(&f, "18523402010209090909090909090909090909090909");
    }

    #[test]
    fn test_retire_connection_id() {
        let f = Frame::RetireConnectionId {
            sequence_number: 0x1234,
        };

        enc_dec(&f, "195234");
    }

    #[test]
    fn test_path_challenge() {
        let f = Frame::PathChallenge { data: [9; 8] };

        enc_dec(&f, "1a0909090909090909");
    }

    #[test]
    fn test_path_response() {
        let f = Frame::PathResponse { data: [9; 8] };

        enc_dec(&f, "1b0909090909090909");
    }

    #[test]
    fn test_connection_close() {
        let mut f = Frame::ConnectionClose {
            close_type: CloseType::Transport,
            error_code: 0x5678,
            frame_type: 0x1234,
            reason_phrase: vec![0x01, 0x02, 0x03],
        };

        enc_dec(&f, "1c5678523403010203");

        f = Frame::ConnectionClose {
            close_type: CloseType::Application,
            error_code: 0x5678,
            frame_type: 0x1234,
            reason_phrase: vec![0x01, 0x02, 0x03],
        };

        enc_dec(&f, "1d5678523403010203");
    }

    #[test]
    fn test_compare() {
        let f1 = Frame::Padding;
        let f2 = Frame::Padding;
        let f3 = Frame::Crypto {
            offset: 0,
            data: vec![1, 2, 3],
        };
        let f4 = Frame::Crypto {
            offset: 0,
            data: vec![1, 2, 3],
        };
        let f5 = Frame::Crypto {
            offset: 1,
            data: vec![1, 2, 3],
        };
        let f6 = Frame::Crypto {
            offset: 0,
            data: vec![1, 2, 4],
        };

        assert_eq!(f1, f2);
        assert_ne!(f1, f3);
        assert_eq!(f3, f4);
        assert_ne!(f3, f5);
        assert_ne!(f3, f6);
    }

}
