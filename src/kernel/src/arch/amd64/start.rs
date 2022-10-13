use alloc::vec::Vec;
use limine::{
    LimineBootInfoRequest, LimineEntryPointRequest, LimineFile, LimineFramebufferRequest,
    LimineKernelFileRequest, LimineMemoryMapEntryType, LimineMmapRequest, LimineModuleRequest,
    LiminePtr, LimineRsdpRequest,
};

use x86_64::{PhysAddr, VirtAddr};

use crate::{
    initrd::BootModule,
    memory::{MemoryRegion, MemoryRegionKind},
    BootInfo,
};

struct LimineBootInfo {
    //arch: &'static StivaleStruct,
    kernel: &'static LimineFile,
    maps: Vec<MemoryRegion>,
    modules: Vec<BootModule>,
    rsdp: Option<u64>,
}

pub enum BootInfoSystemTable {
    Rsdp,
    Efi,
}

impl BootInfo for LimineBootInfo {
    fn memory_regions(&self) -> &'static [MemoryRegion] {
        unsafe { core::intrinsics::transmute(&self.maps[..]) }
    }

    fn get_modules(&self) -> &'static [BootModule] {
        unsafe { core::intrinsics::transmute(&self.modules[..]) }
    }

    fn kernel_image_info(&self) -> (VirtAddr, usize) {
        (
            VirtAddr::new(self.kernel.base.as_ptr().unwrap() as u64),
            self.kernel.length as usize,
        )
    }

    fn get_system_table(&self, table: BootInfoSystemTable) -> VirtAddr {
        match table {
            BootInfoSystemTable::Rsdp => VirtAddr::new(self.rsdp.unwrap()),
            BootInfoSystemTable::Efi => todo!(),
        }
    }

    fn get_cmd_line(&self) -> &'static str {
        if let Some(cmd) = self.kernel.cmdline.as_ptr() {
            let ptr = cmd as *const u8;
            let slice = unsafe { core::slice::from_raw_parts(ptr, 0x1000) };
            let slice = &slice[0..slice.iter().position(|r| *r == 0).unwrap_or(0)];
            core::str::from_utf8(slice).unwrap()
        } else {
            ""
        }
    }
}

impl From<LimineMemoryMapEntryType> for MemoryRegionKind {
    fn from(st: LimineMemoryMapEntryType) -> Self {
        match st {
            LimineMemoryMapEntryType::Usable => MemoryRegionKind::UsableRam,
            // LimineMemoryMapEntryType::AcpiReclaimable => ,
            // LimineMemoryMapEntryType::AcpiNvs => todo!(),
            // LimineMemoryMapEntryType::BadMemory => todo!(),
            // LimineMemoryMapEntryType::BootloaderReclaimable => todo!(),
            LimineMemoryMapEntryType::KernelAndModules => MemoryRegionKind::BootloaderReserved,
            //LimineMemoryMapEntryType::Framebuffer => todo!(),
            _ => MemoryRegionKind::Reserved,
            /*
            StivaleMemoryMapEntryType::Usable => MemoryRegionKind::UsableRam,
            StivaleMemoryMapEntryType::BootloaderReclaimable => {
                MemoryRegionKind::BootloaderReserved
            }
            StivaleMemoryMapEntryType::Kernel => MemoryRegionKind::BootloaderReserved,
            _ => MemoryRegionKind::Reserved,
            */
        }
    }
}

/*
extern "C" fn __stivale_start(info: &'static StivaleStruct) -> ! {
    unsafe {
        let efer = x86::msr::rdmsr(x86::msr::IA32_EFER);
        x86::msr::wrmsr(x86::msr::IA32_EFER, efer | (1 << 11));
        let cr4 = x86::controlregs::cr4();
        x86::controlregs::cr4_write(cr4 | x86::controlregs::Cr4::CR4_ENABLE_GLOBAL_PAGES);
    }
    let mut boot_info = StivaleBootInfo {
        arch: info,
        maps: alloc::vec![],
        modules: alloc::vec![],
        cmd: None,
    };
    boot_info.cmd = info.command_line().map(|cmd| cmd.command_line);
    boot_info.maps = info
        .memory_map()
        .expect("no memory map passed from bootloader")
        .iter()
        .map(|m| MemoryRegion {
            kind: m.entry_type().into(),
            start: PhysAddr::new(m.base),
            length: m.length as usize,
        })
        .collect();
    boot_info.modules = info
        .modules()
        .expect("no modules specified for kernel --- no way to start init")
        .iter()
        .map(|m| BootModule {
            start: VirtAddr::new(m.start),
            length: m.size() as usize,
        })
        .collect();
    crate::kernel_main(&mut boot_info);
}

#[link_section = ".stivale2hdr"]
#[used]
#[no_mangle]
static STIVALE_HDR: StivaleHeader = StivaleHeader::new()
    .entry_point(__stivale_start)
    .stack(&STACK.0[STACK_SIZE - 4096] as *const u8)
    .tags((&FRAMEBUFFER_TAG as *const StivaleFramebufferHeaderTag).cast());

static UNMAP_NULL: StivaleUnmapNullHeaderTag = StivaleUnmapNullHeaderTag::new();

static FRAMEBUFFER_TAG: StivaleFramebufferHeaderTag = StivaleFramebufferHeaderTag::new()
    .framebuffer_bpp(24)
    .next((&UNMAP_NULL as *const StivaleUnmapNullHeaderTag).cast());

    */
const STACK_SIZE: usize = 4096 * 16;
#[repr(C, align(4096))]
struct P2Align12<T>(T);
static STACK: P2Align12<[u8; STACK_SIZE]> = P2Align12([0; STACK_SIZE]);

fn limine_entry() -> ! {
    unsafe {
        let efer = x86::msr::rdmsr(x86::msr::IA32_EFER);
        x86::msr::wrmsr(x86::msr::IA32_EFER, efer | (1 << 11));
        let cr4 = x86::controlregs::cr4();
        x86::controlregs::cr4_write(cr4 | x86::controlregs::Cr4::CR4_ENABLE_GLOBAL_PAGES);
        let cr0 = x86::controlregs::cr0();
        x86::controlregs::cr0_write(cr0 & !x86::controlregs::Cr0::CR0_WRITE_PROTECT);
    }

    LIMINE_BOOTINFO.get_response().get().unwrap();

    let mut boot_info = LimineBootInfo {
        kernel: unsafe {
            LIMINE_KERNEL
                .get_response()
                .get()
                .expect("no kernel info specified for kernel")
                .kernel_file
                .as_ptr()
                .unwrap()
                .as_ref()
                .unwrap()
        },
        maps: alloc::vec![],
        modules: alloc::vec![],
        rsdp: LIMINE_TABLE.get_response().get().map(|r| {
            r.address.as_ptr().unwrap() as u64 - 0xffff800000000000
        } /* TODO: MEGA HACK */),
    };

    boot_info.maps = LIMINE_MEM
        .get_response()
        .get()
        .expect("no memory map specified for kernel")
        .mmap()
        .unwrap()
        .iter()
        .map(|m| MemoryRegion {
            kind: m.typ.into(),
            start: PhysAddr::new(m.base),
            length: m.len as usize,
        })
        .collect();
    boot_info.modules = LIMINE_MOD
        .get_response()
        .get()
        .expect("no modules specified for kernel -- no way to start init")
        .modules()
        .expect("no modules specified for kernel -- no way to start init")
        .iter()
        .map(|m| BootModule {
            start: VirtAddr::new(unsafe { m.base.as_ptr().unwrap() as u64 }),
            length: m.length as usize,
        })
        .collect();
    crate::kernel_main(&mut boot_info);
}

static LIMINE_BOOTINFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
static LIMINE_ENTRY: LimineEntryPointRequest =
    LimineEntryPointRequest::new(0).entry(LiminePtr::new(limine_entry));
//#[link_section = ".data"]
static LIMINE_FB: LimineFramebufferRequest = LimineFramebufferRequest::new(0);
static LIMINE_MOD: LimineModuleRequest = LimineModuleRequest::new(0);
//#[link_section = ".data"]
static LIMINE_MEM: LimineMmapRequest = LimineMmapRequest::new(0);
//#[link_section = ".data"]
static LIMINE_KERNEL: LimineKernelFileRequest = LimineKernelFileRequest::new(0);
static LIMINE_TABLE: LimineRsdpRequest = LimineRsdpRequest::new(0);

#[link_section = ".limine_reqs"]
#[used]
static F1: &'static LimineBootInfoRequest = &LIMINE_BOOTINFO;
#[link_section = ".limine_reqs"]
#[used]
static F2: &'static LimineEntryPointRequest = &LIMINE_ENTRY;
#[link_section = ".limine_reqs"]
#[used]
static F3: &'static LimineModuleRequest = &LIMINE_MOD;
#[link_section = ".limine_reqs"]
#[used]
static F4: &'static LimineMmapRequest = &LIMINE_MEM;
#[link_section = ".limine_reqs"]
#[used]
static F5: &'static LimineKernelFileRequest = &LIMINE_KERNEL;
#[link_section = ".limine_reqs"]
#[used]
static F6: &'static LimineFramebufferRequest = &LIMINE_FB;
#[link_section = ".limine_reqs"]
#[used]
static F7: &'static LimineRsdpRequest = &LIMINE_TABLE;
#[link_section = ".limine_reqs"]
#[used]
static FEND: u64 = 0;
