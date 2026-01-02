### Example

```rust
use garde::Validate;

use rusqlite_derive::{FromSql, ToSql};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Validate, FromSql, ToSql)]
#[garde(transparent)]
#[serde(transparent)]
pub struct Username(
    #[garde(length(min = 3, max = 24), alphanumeric)]
    //
    String,
);
```

```rust
use rusqlite_derive::FromSql;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, FromSql, ToSql)]
#[serde(try_from = String)]
#[rusqlite(try_from = String)]
pub struct Username(String);

impl TryFrom<String> for Username {
    type Error = UsernameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() >= 3 && value.len() <= 24 && value.chars().all(|c| c.is_alphanumeric()) {
            Ok(Self(value))
        } else {
            Err(UsernameError::Invalid)
        }
    }
}
```
