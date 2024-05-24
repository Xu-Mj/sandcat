use sandcat_sdk::state::MobileState;

pub mod action;
pub mod dialog;
pub mod left;
pub mod phone_call;
pub mod right;
pub mod select_friends;
pub mod self_info;
pub mod top_bar;

pub fn get_platform(is_mobile: bool) -> i32 {
    if is_mobile {
        MobileState::Mobile as i32
    } else {
        MobileState::Desktop as i32
    }
}
