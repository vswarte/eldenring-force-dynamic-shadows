use std::mem;
use std::sync;
use broadsword::runtime::get_module_handle;
use detour::static_detour;

use broadsword::dll;

static GAME_BASE: sync::OnceLock<usize> = sync::OnceLock::new();

const HAS_DYNAMIC_SHADOW_CAST_IBO: usize = 0x1BED82E;
const GXLIGHTMANAGER_CONSTRUCTOR_IBO: usize = 0x1A255A0;

static_detour! {
    static GXLIGHTMANAGER_CONSTRUCTOR: unsafe extern "system" fn(u64, u64) -> u64;
}

#[dll::entrypoint]
pub fn entry(_: usize) -> bool {
    apply_hook();
    true
}

fn apply_hook() {
    unsafe {
        GXLIGHTMANAGER_CONSTRUCTOR
            .initialize(
                mem::transmute(get_game_base() + GXLIGHTMANAGER_CONSTRUCTOR_IBO),
                |allocated_space: u64, param_2: u64| {
                    let result = GXLIGHTMANAGER_CONSTRUCTOR.call(allocated_space, param_2);
                    patch_dynamic_shadow_check();
                    result
                }
            )
            .unwrap();

        GXLIGHTMANAGER_CONSTRUCTOR.enable().unwrap();
    }
}

// Overwrites the value that the dynamic shadow setting on a light source is compared to.
// By setting it to 0x2 it'll always apply regardless of the setting (either 0x0 or 0x1).
fn patch_dynamic_shadow_check() {
    let ptr = get_game_base() + HAS_DYNAMIC_SHADOW_CAST_IBO;

    unsafe { *(ptr as *mut u8) = 0x2 };
}

pub fn get_game_base() -> usize {
    *GAME_BASE.get_or_init(|| {
        get_module_handle("eldenring.exe")
            .or_else(|_| get_module_handle("start_protected_game.exe"))
            .expect("Could not locate main module")
    })
}
