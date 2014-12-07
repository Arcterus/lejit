#![experimental]

use libc;
use std::ptr;
use std::os;

#[experimental]
pub trait MemoryRegion {
   fn protect(&mut self) -> bool;
   fn copy(&mut self, data: &[u8]) -> bool;
}

#[experimental]
impl MemoryRegion for os::MemoryMap {
   fn protect(&mut self) -> bool {
      unsafe {
         libc::mprotect(self.data() as *mut libc::c_void,
                        self.len() as libc::size_t,
                        libc::PROT_READ | libc::PROT_EXEC) == -1
      }
   }

   fn copy(&mut self, data: &[u8]) -> bool {
      if data.len() > self.len() {
         false
      } else {
         unsafe {
            ptr::copy_memory(self.data(), data.as_ptr(), data.len());
         }
         true
      }
   }
}
