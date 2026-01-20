use std::ffi::{CString};
use std::os::raw::{c_char, c_int};

extern "C" {
    fn nvm_assemble_from_str(asm_source: *const c_char, output_path: *const c_char) -> c_int;
}

pub fn assemble_from_str(asm_code: &str, output_path: &str) -> Result<(), String> {
    let c_asm = CString::new(asm_code).map_err(|_| "ASM code contained null byte".to_string())?;
    let c_out = CString::new(output_path).map_err(|_| "Output path contained null byte".to_string())?;
    let rc = unsafe { nvm_assemble_from_str(c_asm.as_ptr(), c_out.as_ptr()) };
    if rc == 0 { Ok(()) } else { Err(format!("C assembler returned error code {}", rc)) }
}
