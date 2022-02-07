# jsave
Persistent storage copy in JSON format for serializable in-memory data

[![Version](https://img.shields.io/crates/v/jsave.svg?style=flat)](https://crates.io/crates/jsave)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/jsave)
[![License](https://img.shields.io/crates/l/jsave.svg?style=flat)](https://github.com/EAimTY/jsave/blob/master/LICENSE)

## Design
Just like `RwLock`, jsave is a reader-writer lock, but serializes and saves data to a file in json format on every writing operation finish

**Do not use it unless you only want to persist a tiny amount of data**

## Usage
```rust
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

// Data to be persisted. Needs to be serializable and deserializable
#[derive(Debug, Serialize, Deserialize)]
struct Data {
    data: HashMap<String, usize>,
}

let data = Data {
    data: HashMap::new(),
};

use std::fs::File;

// Create the file for storing the data
File::create("db_file").unwrap();

use jsave::Jsave;

// Initialize a new Jsave instance with the given data. Note that the file will be truncated
let db = Jsave::init_with(data, "db_file").unwrap();

{
    // Read data
    let db_read = db.read();
    println!("{:?}", *db_read);
}

{
    // Read and write data
    let mut db_write = db.write();
    db_write.data.insert("foo".to_owned(), 114514);
    println!("{:?}", *db_write);

    // Automatically saving data to file when the `JsaveWriteGuard` is dropped
}

drop(db);

// Initialize a new Jsave instance from a file
let db = Jsave::<Data>::init("db_file").unwrap();

let db_read = db.read();
println!("{:?}", *db_read);
```

## License
GNU General Public License v3.0
