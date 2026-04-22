use wgpu::{Device, Queue, Adapter, Instance};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Failed to request adapter")]
    NoAdapter,
    #[error("Failed to request device: {0}")]
    DeviceRequest(String),
    #[error("Failed to find suitable adapter")]
    NoSuitableAdapter,
}

pub struct RenderDevice {
    pub device: Device,
    pub queue: Queue,
    pub adapter: Adapter,
    pub instance: Instance,
}

impl RenderDevice {
    pub async fn new() -> Result<Self, DeviceError> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(DeviceError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RETAS Render Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| DeviceError::DeviceRequest(e.to_string()))?;

        Ok(Self {
            device,
            queue,
            adapter,
            instance,
        })
    }

    pub async fn with_surface(surface: &wgpu::Surface<'_>) -> Result<Self, DeviceError> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(DeviceError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RETAS Render Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| DeviceError::DeviceRequest(e.to_string()))?;

        Ok(Self {
            device,
            queue,
            adapter,
            instance,
        })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }
}
