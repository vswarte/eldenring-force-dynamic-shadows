use std::slice;
use region::Protection;
use broadsword::runtime;
use broadsword::scanner;
use broadsword::dll;


const HAS_DYNAMIC_SHADOW_CAST_PATTERN: &str = concat!(
    // 141ba1ad8 80 bf ad        CMP        byte ptr [RDI + 0xad],0x0
    //           00 00 00 00
    "10000000 10111111 10101101 00000000 00000000 00000000 00000000",
    // 141ba1adf 74 1f           JZ         LAB_141ba1b00
    "01110100 ........",
    // 141ba1ae1 0f 2f cf        COMISS     XMM1,XMM7
    "00001111 00101111 11001111",
    // 141ba1ae4 76 1a           JBE        LAB_141ba1b00
    "01110110 ........",
    // 141ba1ae6 41 80 7e        CMP        byte ptr [R14 + 0x1],0x0
    //           01 00
    "01000... 10000000 01111110 00000001 00000000",
    // 141ba1aeb 74 13           JZ         LAB_141ba1b00
    "01110100 ........",
    // 141ba1aed 41 8b 46 04     MOV        EAX,dword ptr [R14 + 0x4]
    "01000... 10001011 01000110 00000100",
    // 141ba1af1 39 87 b8        CMP        dword ptr [RDI + 0xb8],EAX
    //           00 00 00
    "........ ..000111 10111000 00000000 00000000 00000000",
    // 141ba1af7 77 07           JA         LAB_141ba1b00
    "01110111 ........",
    // 141ba1af9 b8 01 00        MOV        EAX,0x1
    //           00 00
    "10111000 00000001 00000000 00000000 00000000",
    // 141ba1afe eb 02           JMP        LAB_141ba1b02
    "11101011 ........",
);

#[dll::entrypoint]
pub fn entry(_: usize) -> bool {
    let shadow_cmp = (
        match_instruction_pattern(HAS_DYNAMIC_SHADOW_CAST_PATTERN).unwrap() + 0x6
    ) as *mut u8;

    unsafe {
        let page_start = (
            shadow_cmp as usize / region::page::size()
        ) * region::page::size();

        let _handle = region::protect_with_handle(
            page_start as *const u8,
            region::page::size(),
            Protection::READ_WRITE_EXECUTE,
        ).unwrap();

        *shadow_cmp = 0x2;
    }

    true
}

pub fn match_instruction_pattern(pattern: &str) -> Option<usize> {
    let text_section = runtime::get_module_section_range("eldenring.exe", ".text")
        .or_else(|_| runtime::get_module_section_range("start_protected_game.exe", ".text"))
        .unwrap();

    let scan_slice = unsafe {
        slice::from_raw_parts(
            text_section.start as *const u8,
            text_section.end - text_section.start,
        )
    };

    let pattern = scanner::Pattern::from_bit_pattern(pattern).unwrap();

    scanner::simple::scan(scan_slice, &pattern)
        .map(|r| text_section.start + r.location)
}
