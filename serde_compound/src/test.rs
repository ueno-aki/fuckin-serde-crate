use bytes::{BufMut, BytesMut};
use proto_bytes::*;
use serde::{Deserialize, Serialize};

use crate::{from_buffer, types::NBTTypes};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: i32,
    name: String,
    pos: Position,
    bytes: Vec<i64>,
    package: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Position {
    x: f64,
    z: f64,
}

#[test]
fn deserialize_works() {
    let user: User = from_buffer(&create_buf()).unwrap();
    assert_eq!(
        user,
        User {
            id: 45,
            name: "Mark2".to_string(),
            pos: Position { x: 23.1, z: 5.6 },
            bytes: vec![12345679, 2345679, 345679, -345679, -45679, -5679],
            package: vec![
                "Shut".to_string(),
                "your".to_string(),
                "fuckin'".to_string(),
                "mouth".to_string()
            ]
        }
    )
}

fn create_buf() -> Vec<u8> {
    let mut vec = BytesMut::new();
    vec.put_i8(NBTTypes::Compound as i8);
    vec.put_short_string("user");

    vec.put_i8(NBTTypes::Int as i8);
    vec.put_short_string("id");
    vec.put_i32_le(45);

    vec.put_i8(NBTTypes::LongArray as i8);
    vec.put_short_string("bytes");
    vec.put_i32_le(6);
    vec.put_i64_le(12345679);
    vec.put_i64_le(2345679);
    vec.put_i64_le(345679);
    vec.put_i64_le(-345679);
    vec.put_i64_le(-45679);
    vec.put_i64_le(-5679);

    vec.put_i8(NBTTypes::List as i8);
    vec.put_short_string("package");
    vec.put_i8(NBTTypes::String as i8);
    vec.put_i32_le(4);
    vec.put_short_string("Shut");
    vec.put_short_string("your");
    vec.put_short_string("fuckin'");
    vec.put_short_string("mouth");

    //dummy
    vec.put_i8(NBTTypes::List as i8);
    vec.put_short_string("dummy");
    vec.put_i8(NBTTypes::String as i8);
    vec.put_i32_le(4);
    vec.put_short_string("Shut");
    vec.put_short_string("your");
    vec.put_short_string("fuckin'");
    vec.put_short_string("mouth");

    vec.put_i8(NBTTypes::Compound as i8);
    vec.put_short_string("dummmy");
    vec.put_i8(NBTTypes::Double as i8);
    vec.put_short_string("x");
    vec.put_f64_le(23.1);
    vec.put_i8(NBTTypes::Double as i8);
    vec.put_short_string("z");
    vec.put_f64_le(5.6);
    vec.put_i8(NBTTypes::Void as i8);
    
    vec.put_i8(NBTTypes::String as i8);
    vec.put_short_string("dummmydummmy");
    vec.put_short_string("Mark2");
    //

    vec.put_i8(NBTTypes::Compound as i8);
    vec.put_short_string("pos");
    vec.put_i8(NBTTypes::Double as i8);
    vec.put_short_string("x");
    vec.put_f64_le(23.1);
    vec.put_i8(NBTTypes::Double as i8);
    vec.put_short_string("z");
    vec.put_f64_le(5.6);
    vec.put_i8(NBTTypes::Void as i8);

    vec.put_i8(NBTTypes::String as i8);
    vec.put_short_string("name");
    vec.put_short_string("Mark2");

    vec.put_i8(NBTTypes::Void as i8);
    vec.to_vec()
}
