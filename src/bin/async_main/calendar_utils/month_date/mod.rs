#[cfg(not(feature = "monthdate-packed"))]
mod naive;
#[cfg(not(feature = "monthdate-packed"))]
pub use naive::MonthDate;
#[cfg(feature = "monthdate-packed")]
mod packed;
#[cfg(feature = "monthdate-packed")]
pub use packed::MonthDate;
