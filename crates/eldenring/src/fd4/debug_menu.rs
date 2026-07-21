/// Native debug-menu node used by FD4 debug menu registration helpers.
///
/// Static evidence: Elden Ring debug-registration helpers store pointers to
/// objects whose first qword is a debug-menu vtable, for example the root object
/// at `PTR_PTR_143c61160` passed to `GameDataMan` and `CSMenuManImp` debug menu
/// setup when `GLOBAL_FD4DebugMenuManager` exists. Only the vtable header is
/// modeled here because call sites in this crate retain the node as a handle.
#[repr(C)]
pub struct FD4DebugMenuNode {
    vtable: usize,
}
