
use core::ffi::c_void;

/// Execute some function on the target AP (blocking). 
///
/// NOTE: It seems like MpServices only exposes a *blocking* interface 
/// for running code on other APs. 
pub fn smp_call(cpu_num: usize, func: fn()) -> uefi::Result<()> {
    use uefi::proto::pi::mp::MpServices;
    use uefi::boot::{
        get_handle_for_protocol,
        open_protocol_exclusive,
    };
    let handle = get_handle_for_protocol::<MpServices>()?;
    let mp_services = open_protocol_exclusive::<MpServices>(handle)?;
    mp_services.startup_this_ap(
        cpu_num, 
        _do_smp_call, 
        func as *mut c_void, 
        None, 
        None
    )
}

extern "efiapi" fn _do_smp_call(content: *mut core::ffi::c_void) {
    let func: fn() = unsafe { core::mem::transmute(content) };
    func();
}


