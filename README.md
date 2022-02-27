# jsave
Persist serializable in-memory data in JSON format

[![Version](https://img.shields.io/crates/v/jsave.svg?style=flat)](https://crates.io/crates/jsave)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/jsave)
[![License](https://img.shields.io/crates/l/jsave.svg?style=flat)](https://github.com/EAimTY/jsave/blob/master/LICENSE)

**Do not use jsave unless you only have a small amount of data. It is not really IO-efficient. Use a proper database like SQLite instead**

jsave provides `RwLock`, `Mutex` and `ReentrantMutex`, which are wraps of those in [parking_lot](https://github.com/Amanieu/parking_lot), with addition APIs to serialize and store in-memory data to file

## Usage
```rust
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io};

// Data to be persisted. Needs to be serializable and deserializable
#[derive(Debug, Serialize, Deserialize)]
struct Data(HashMap<String, usize>);

impl Default for Data {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

fn main() -> io::Result<()> {
    let path = "PATH_TO_DB_FILE";

    use jsave::Mutex;
    use std::fs::OpenOptions;

    // Open the database file, or create it if it doesn't exist
    let db = if OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .is_ok()
    {
        Mutex::init_with(Data::default(), path)?
    } else {
        Mutex::init(path)?
    };

    {
        // Read and write data just like a regular `Mutex`
        let mut db = db.lock();
        db.0.insert("foo".to_string(), 114514);
        println!("{:?}", *db);
    }

    // Save the data onto the disk. The `Mutex` is locked until the save is complete
    db.save()?;

    Ok(())
}
```

## Optional Features

- `pretty` - Store the data as a pretty-printed String of JSON
- `send_guard` - Allow lock guards to be sent to other threads
- `preserve_order` - Read data into a Value and written back to a JSON string while preserving the order of map keys in the input
- `float_roundtrip` - Use sufficient precision when parsing fixed precision floats from JSON to ensure that they maintain accuracy when round-tripped through JSON. This comes at an approximately 2x performance cost for parsing floats compared to the default best-effort precision
- `arbitrary_precision` - Use an arbitrary precision number representation for serde_json::Number. This allows JSON numbers of arbitrary size/precision to be read into a Number and written back to a JSON string without loss of precision
- `unbounded_depth` - Provide a method disable_recursion_limit to parse arbitrarily deep JSON structures without any consideration for overflowing the stack

## License
GNU General Public License v3.0
