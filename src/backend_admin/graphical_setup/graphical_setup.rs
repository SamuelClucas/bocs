use wgpu::InstanceDescriptor;
use wgpu::{Instance, Adapter, RequestAdapterOptions};

fn init_device_queue(){
    let inst_desc = InstanceDescriptor::from_env_or_default();
    let inst = Instance::new(&inst_desc);
    let rq_adapt_opts: RequestAdapterOptions = Default::default();
    let adapt: Adapter = inst.request_adapter(&rq_adapt_opts);
}
