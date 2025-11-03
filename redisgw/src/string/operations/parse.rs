use crate::string::operations::{SetFlags, SetMethod, SetTTL};

/// Parse extra arguments for the SET command (NX/XX/GET/EX/PX/EXAT/PXAT/KEEPTTL)
/// Returns a `SetFlags` value representing parsed options.
pub(crate) fn parse_set_extra_args(extra_args: &[&[u8]]) -> SetFlags {
    if extra_args.is_empty() {
        return SetFlags::default();
    }

    let mut method: Option<SetMethod> = None;
    let mut ttl: Option<SetTTL> = None;
    let mut get: bool = false;

    let mut args = extra_args.iter().peekable();
    while let Some(arg) = args.next() {
        let s = match std::str::from_utf8(arg) {
            Ok(s) => s.to_ascii_uppercase(),
            Err(_) => continue,
        };
        match s.as_str() {
            "NX" => method = Some(SetMethod::NX),
            "XX" => method = Some(SetMethod::XX),
            "GET" => get = true,
            "EX" | "PX" | "EXAT" | "PXAT" => {
                let next = match args.next() {
                    Some(n) => n,
                    None => continue,
                };
                let n = match std::str::from_utf8(next).ok().and_then(|s| s.parse::<u64>().ok()) {
                    Some(val) => val,
                    None => continue,
                };
                ttl = Some(match s.as_str() {
                    "EX" => SetTTL::Ex(n),
                    "PX" => SetTTL::Px(n),
                    "EXAT" => SetTTL::ExAt(n),
                    "PXAT" => SetTTL::PxAt(n),
                    _ => unreachable!(),
                });
            }
            "KEEPTTL" => ttl = Some(SetTTL::KeepTTL),
            _ => {}
        }
    }

    SetFlags { method, ttl, get }
}
