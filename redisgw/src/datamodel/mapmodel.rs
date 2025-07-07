use foundationdb_tuple::{TupleDepth, TuplePack, VersionstampOffset};
use std::io::Write;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataPrefix {
    Data = 21,
    KeyTtl = 22,
    FieldTtl = 23,
}

impl TuplePack for DataPrefix {
    fn pack<W: Write>(
        &self,
        w: &mut W,
        tuple_depth: TupleDepth,
    ) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}

struct MapDataModel {}
