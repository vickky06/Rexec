pub mod executor {
    tonic::include_proto!("executor");

    // Include the file descriptor set for reflection
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("executor_descriptor");
}