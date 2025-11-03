use crate::command::CmdMap;


/// Return handlers for set-related commands (SADD, SREM)
pub fn commands(
) -> CmdMap {
  let m: CmdMap = CmdMap::new();
  m
}
