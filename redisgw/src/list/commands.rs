use crate::command::CmdMap;


/// Return handlers for list-related commands (LPUSH, RPUSH)
pub fn commands(
) -> CmdMap {
  let m: CmdMap = CmdMap::new();
  m
}
