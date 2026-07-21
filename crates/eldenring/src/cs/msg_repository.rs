/// Native `CS::MsgRepositoryImp` singleton prefix.
///
/// Source of name: singleton symbol. This type intentionally models only the
/// known vtable prefix; the remaining native object tail is not modeled until
/// its field layout is reversed.
#[shared::singleton("MsgRepository")]
#[repr(C)]
pub struct MsgRepositoryImp {
    vftable: usize,
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use super::MsgRepositoryImp;

    #[test]
    fn layout_offsets_match_static_re() {
        assert_eq!(0x0, offset_of!(MsgRepositoryImp, vftable));
        assert_eq!(0x8, size_of::<MsgRepositoryImp>());
    }
}
