pub mod services;

pub mod pb {
    #[allow(ambiguous_associated_items)]
    pub mod pipeline {
        tonic::include_proto!("photopipeline.pipeline");
    }
    #[allow(ambiguous_associated_items)]
    pub mod image {
        tonic::include_proto!("photopipeline.image");
    }
    #[allow(ambiguous_associated_items)]
    pub mod batch {
        tonic::include_proto!("photopipeline.batch");
    }
}

#[derive(Default)]
pub struct SharedState {}
