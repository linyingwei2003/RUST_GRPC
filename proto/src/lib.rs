pub mod greet {
    tonic::include_proto!("greet");
}

pub mod ads {
    tonic::include_proto!("ads");
}

pub use greet::*;
pub use ads::*;
