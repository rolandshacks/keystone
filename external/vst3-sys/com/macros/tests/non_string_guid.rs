use com::com_interface;
use com::interfaces::iunknown::IUnknown;

#[com_interface(cc2d05c7-7d20-4ccb-ad75-1e7fb7c77254)]
pub trait Interface: IUnknown {}

fn main() {}
