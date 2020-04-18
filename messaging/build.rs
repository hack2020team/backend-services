fn main() {
    tonic_build::compile_protos("../vendor/proto/hack2020team/headposeservice/pose_service.proto").unwrap();
}
