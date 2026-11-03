use rf24::radio::prelude::{
    EsbAutoAck, EsbFifo, EsbPayloadLength, EsbPipe, EsbRadio, RadioErrorType,
};

use crate::{addresses::is_valid_address, frame::RF24NetworkFrame};

pub trait NetworkErrorType<RadioType> {
    type Error;
}

pub trait StaticNetwork<RadioType>: NetworkErrorType<RadioType>
where
    RadioType: EsbRadio + EsbAutoAck + EsbPayloadLength + EsbFifo + EsbPipe,
{
    fn update(&mut self, radio: &mut RadioType) -> Result<u8, Self::Error>
    where
        <Self as NetworkErrorType<RadioType>>::Error: From<<RadioType as RadioErrorType>::Error>,
    {
        let mut keep_going = true;
        while keep_going {
            if radio.available()? {
                let mut frame_buf = [0; 32];
                let len = radio.read(&mut frame_buf, None)?;
                if let Some(frame) = RF24NetworkFrame::from_buffer(&frame_buf) {
                    if !is_valid_address(frame.header.from_node) || !is_valid_address(frame.header.to_node){
                        continue;
                    }
                }
            } else {
                break;
            }
        }
        Ok(0)
    }

    fn send(&mut self, radio: &mut RadioType, to_node: u16) -> Result<bool, Self::Error>;
    fn read(
        &mut self,
        radio: &mut RadioType,
        frame: &mut RF24NetworkFrame,
    ) -> Result<(), Self::Error>;
}
