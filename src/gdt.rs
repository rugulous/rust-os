use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{Segment, CS};
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code);
        load_tss(GDT.1.tss);
    }
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };

    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {code: code_selector, tss: tss_selector})
    };
}

struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector
}