use std::sync::{LazyLock, atomic::AtomicUsize};

use chrono::Utc;

use crate::services::core::classes::Ref;

static COUNT: AtomicUsize = AtomicUsize::new(0);
static RANDOM: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{:6X}{:4X}",
        rand::random::<u32>().wrapping_mul(1 << 24),
        rand::random::<u32>().wrapping_mul(1 << 16)
    )
});

pub fn generate_object_id() -> Ref {
    let count = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut timestamp = Utc::now().timestamp() / 1000;
    if timestamp < 0 {
        timestamp = 0;
    }

    format!("{timestamp:08X}{}{count}", &*RANDOM)
}
