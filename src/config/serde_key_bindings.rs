use piston::input;
use serde::{
    de::{Deserializer, Error, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum InputButton {
    Keyboard {
        value: u32,
    },
    Mouse {
        value: u32,
    },
    Controller {
        id: i32,
        button: u8,
    },
    Hat {
        id: i32,
        state: input::HatState,
        which: u8,
    },
}

impl From<InputButton> for input::Button {
    fn from(button: InputButton) -> Self {
        match button {
            InputButton::Keyboard { value } => input::Button::Keyboard(value.into()),
            InputButton::Mouse { value } => input::Button::Mouse(value.into()),
            InputButton::Controller { id, button } => {
                input::Button::Controller(input::controller::ControllerButton { id, button })
            }
            InputButton::Hat { id, state, which } => {
                input::Button::Hat(input::controller::ControllerHat { id, state, which })
            }
        }
    }
}

pub fn serialize<S>(buttons: &[input::Button; 7], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(7))?;
    for button in buttons {
        match *button {
            input::Button::Keyboard(key) => {
                seq.serialize_element(&InputButton::Keyboard { value: key.into() })
            }
            input::Button::Mouse(button) => seq.serialize_element(&InputButton::Mouse {
                value: button.into(),
            }),
            input::Button::Controller(c) => seq.serialize_element(&InputButton::Controller {
                id: c.id,
                button: c.button,
            }),
            input::Button::Hat(hat) => seq.serialize_element(&InputButton::Hat {
                id: hat.id,
                state: hat.state,
                which: hat.which,
            }),
        }?;
    }
    seq.end()
}

pub fn deserialize<'de, D>(d: D) -> Result<[input::Button; 7], D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_seq(KeyBindingsVisitor)
}
struct KeyBindingsVisitor;

impl<'de> Visitor<'de> for KeyBindingsVisitor {
    type Value = [input::Button; 7];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an array of 7 button descriptors")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(len) = seq.size_hint() {
            if len != 7 {
                return Err(A::Error::invalid_length(len, &self));
            }
        }
        let keys = [
            seq.next_element::<InputButton>()?
                .ok_or(A::Error::invalid_length(0, &self))?
                .into(),
            seq.next_element::<InputButton>()?
                .ok_or(A::Error::invalid_length(1, &self))?
                .into(),
            seq.next_element::<InputButton>()?
                .ok_or(A::Error::invalid_length(2, &self))?
                .into(),
            seq.next_element::<InputButton>()?
                .ok_or(A::Error::invalid_length(3, &self))?
                .into(),
            seq.next_element::<InputButton>()?
                .ok_or(A::Error::invalid_length(4, &self))?
                .into(),
            seq.next_element::<InputButton>()?
                .ok_or(A::Error::invalid_length(5, &self))?
                .into(),
            seq.next_element::<InputButton>()?
                .ok_or(A::Error::invalid_length(6, &self))?
                .into(),
        ];

        let mut len = 7;
        while seq.next_element::<InputButton>()?.is_some() {
            len += 1;
        }
        if len != 7 {
            Err(A::Error::invalid_length(len, &self))
        } else {
            Ok(keys)
        }
    }
}
