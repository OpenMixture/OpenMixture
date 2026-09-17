//! Diagnostic only: execute unchanged WGSL kernels and retain each rgba16float texture.
use mixture_wgpu::{GpuContext,GpuContextOptions,BackendPreference};
fn main(){
 let args:Vec<String>=std::env::args().collect(); let nodes:serde_json::Value=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap(); std::fs::create_dir_all(&args[2]).unwrap();
 let context=pollster::block_on(GpuContext::request(GpuContextOptions{backend:BackendPreference::Dx12,..Default::default()})).unwrap();
 std::fs::write(format!("{}/context.json",args[2]),serde_json::to_vec_pretty(context.report()).unwrap()).unwrap();
 let (device,queue)=(context.device(),context.queue()); let mut textures:Vec<wgpu::Texture>=vec![];
 for node in nodes.as_array().unwrap(){
 let texture=device.create_texture(&wgpu::TextureDescriptor{label:None,size:wgpu::Extent3d{width:1024,height:1024,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba16Float,usage:wgpu::TextureUsages::STORAGE_BINDING|wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
 let uniform=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:(node["params"].as_array().unwrap().len()*4) as u64,usage:wgpu::BufferUsages::UNIFORM,mapped_at_creation:true});
 let bytes:Vec<u8>=node["params"].as_array().unwrap().iter().flat_map(|v|(v.as_u64().unwrap() as u32).to_le_bytes()).collect(); uniform.slice(..).get_mapped_range_mut().unwrap().copy_from_slice(&bytes); uniform.unmap();
 let module=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:None,source:wgpu::ShaderSource::Wgsl(node["code"].as_str().unwrap().into())});
 let pipeline=device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{label:None,layout:None,module:&module,entry_point:node["entry"].as_str(),compilation_options:Default::default(),cache:None});
 let view=texture.create_view(&Default::default()); let inputs:Vec<_>=node["inputs"].as_array().unwrap().iter().map(|i|textures[i.as_u64().unwrap() as usize].create_view(&Default::default())).collect();
 let mut entries=vec![wgpu::BindGroupEntry{binding:0,resource:uniform.as_entire_binding()},wgpu::BindGroupEntry{binding:1,resource:wgpu::BindingResource::TextureView(&view)}];
 for (i,v) in inputs.iter().enumerate(){entries.push(wgpu::BindGroupEntry{binding:i as u32+2,resource:wgpu::BindingResource::TextureView(v)});}
 let group=device.create_bind_group(&wgpu::BindGroupDescriptor{label:None,layout:&pipeline.get_bind_group_layout(0),entries:&entries});
 let staging=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:8388608,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
 let mut encoder=device.create_command_encoder(&Default::default());{let mut pass=encoder.begin_compute_pass(&Default::default());pass.set_pipeline(&pipeline);pass.set_bind_group(0,&group,&[]);pass.dispatch_workgroups(128,128,1);}
 encoder.copy_texture_to_buffer(texture.as_image_copy(),wgpu::TexelCopyBufferInfo{buffer:&staging,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(8192),rows_per_image:Some(1024)}},wgpu::Extent3d{width:1024,height:1024,depth_or_array_layers:1}); queue.submit([encoder.finish()]);
 let(tx,rx)=std::sync::mpsc::channel();staging.slice(..).map_async(wgpu::MapMode::Read,move|r|tx.send(r).unwrap());device.poll(wgpu::PollType::Wait{submission_index:None,timeout:Some(std::time::Duration::from_secs(60))}).unwrap();rx.recv().unwrap().unwrap();std::fs::write(format!("{}/{}.bin",args[2],node["id"].as_str().unwrap()),&*staging.slice(..).get_mapped_range().unwrap()).unwrap();staging.unmap();textures.push(texture);
 }
}
