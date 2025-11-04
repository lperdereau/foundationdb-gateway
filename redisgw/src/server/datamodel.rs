use fdb::FoundationDB;
use foundationdb_tuple::{pack, TuplePack, TupleDepth, VersionstampOffset};
use std::io::Write;
use bcrypt::{hash, DEFAULT_COST};
use foundationdb::RangeOption;
use futures_util::StreamExt;
use futures_util::TryStreamExt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserInfo {
    pub hash: Vec<u8>,
    pub rules: Option<String>,
}

pub struct AuthDataModel {
    pub fdb: FoundationDB,
}

impl AuthDataModel {
    pub fn new(fdb: FoundationDB) -> Self {
        Self { fdb }
    }

    /// Get stored user info (hash + rules) for a username, if any.
    pub async fn get_user(&self, username: &[u8]) -> Result<Option<UserInfo>, String> {
        let key = pack(&(AuthPrefix::User, username));
        let res = self
            .fdb
            .get(&key)
            .await
            .map_err(|e| format!("FoundationDB get error: {:?}", e))?;
        if let Some(bytes) = res {
            // stored format: <hash bytes> '\n' <rules utf8 optional>
            if let Some(pos) = bytes.iter().position(|b| *b == b'\n') {
                let hash = bytes[..pos].to_vec();
                let rules = String::from_utf8_lossy(&bytes[pos + 1..]).into_owned();
                let rules_opt = if rules.is_empty() { None } else { Some(rules) };
                Ok(Some(UserInfo { hash, rules: rules_opt }))
            } else {
                Ok(Some(UserInfo { hash: bytes, rules: None }))
            }
        } else {
            Ok(None)
        }
    }

    /// Create or update a user. `password` may be plaintext or an existing
    /// bcrypt hash. `rules` is an optional utf8 string representing ACL rules.
    pub async fn set_user(&self, username: &[u8], password: &[u8], rules: Option<&[u8]>) -> Result<(), String> {
        let key = pack(&(AuthPrefix::User, username));
        let store_bytes = if password.starts_with(b"$2") {
            password.to_vec()
        } else {
            let pw_str = String::from_utf8_lossy(password);
            let hashed = hash(pw_str.as_ref(), DEFAULT_COST)
                .map_err(|e| format!("bcrypt hash error: {:?}", e))?;
            hashed.into_bytes()
        };

        let mut final_bytes = store_bytes;
        final_bytes.push(b'\n');
        if let Some(r) = rules {
            final_bytes.extend_from_slice(r);
        }

        self.fdb
            .set(&key, &final_bytes)
            .await
            .map_err(|e| format!("FoundationDB set error: {:?}", e))
    }

    pub async fn del_user(&self, username: &[u8]) -> Result<(), String> {
        let key = pack(&(AuthPrefix::User, username));
        self.fdb
            .delete(&key)
            .await
            .map(|_v| ())
            .map_err(|e| format!("FoundationDB delete error: {:?}", e))
    }

    /// List usernames stored in the ACL prefix.
    pub async fn list_users(&self) -> Result<Vec<String>, String> {
        let start = pack(&(AuthPrefix::User, b"" as &[u8]));
        let mut end = start.clone();
        end.push(0xFF);
        let range = RangeOption::from((start.clone(), end));

        let db = self.fdb.clone();
        let res = db
            .database
            .run(move |trx, _| {
                let range = range.clone();
                async move {
                    let stream = trx.get_ranges_keyvalues(range, false);
                    let records = stream
                        .map(|r| match r {
                            Ok(v) => Ok((v.key().to_vec(), v.value().to_vec())),
                            Err(e) => Err(e),
                        })
                        .try_collect::<Vec<(Vec<u8>, Vec<u8>)>>()
                        .await?;

                    let mut out = Vec::new();
                    for (k, _v) in records {
                        out.push(String::from_utf8_lossy(&k).into_owned());
                    }
                    Ok(out)
                }
            })
            .await;

        res.map_err(|e| format!("FoundationDB range error: {:?}", e))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthPrefix {
    User = 21,
}

impl TuplePack for AuthPrefix {
    fn pack<W: Write>(&self, w: &mut W, tuple_depth: TupleDepth) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}
