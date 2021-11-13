use wgpu_engine::*;
use pal::PlatformBackend;
use screen_task::ScreenTask;
use crate::geometry_manager::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::event_processing::WCompMessage;

pub struct WComp {
    pub(crate) timer: std::time::Instant,
    pub(crate) wgpu_engine: wgpu_engine::WGpuEngine,
    pub(crate) screen_task: TaskId,
    pub(crate) platform: pal::Platform,
    pub(crate) ews: ews::EmbeddedWaylandServer,
    pub(crate) geometry_manager: GeometryManager<(),ews::WlSurface,()>,
    pub(crate) messages: Rc<RefCell<Vec<WCompMessage<(),ews::WlSurface,()>>>>,
    //pub(crate) default_cursor: usize
}
impl WComp {
    pub fn new()->Self {

        let features_and_limits = ScreenTask::features_and_limits();

        let mut wgpu_engine = wgpu_engine::WGpuEngine::new(features_and_limits.clone()).expect("Failed to intialize the graphic engine");
        let screen_task = wgpu_engine
            .create_task("ScreenTask".into(), features_and_limits, |_id, _tokio_runtime, update_context| ScreenTask::new(update_context)).unwrap();

        let mut platform = pal::Platform::new(vec![Box::new(wgpu_engine.wgpu_context())]);
        if platform.platform_type() == pal::PlatformType::Compositor {
            platform.request(vec![pal::Request::Surface{request: pal::SurfaceRequest::Create(None)}]);
        }

        let parameters = ews::Parameters {
            shm_formats: Vec::new(),
            drm_formats: vec![
                ews::DrmFormat{code: ews::DrmFourcc::Xrgb8888,modifier: ews::DrmModifier::Linear}
            ]
        };
        let ews = ews::EmbeddedWaylandServer::new(parameters);
        //ews.set_shm_formats(vec![Format::Argb8888,Format::Xrgb8888]);

        let geometry_manager = GeometryManager::new();
        let timer = std::time::Instant::now();
        let messages = Rc::new(RefCell::new(Vec::new()));

        //let icon_path = xcursor::CursorTheme::load("Pop").load_icon("left_ptr").unwrap();
        //println!("Icon path: {:#?}",icon_path);
        //screen_task::SurfaceSource::from_file_path(icon_path);

        /*
        let mut icon_file = File::open(icon_path).ok()?;

        let mut buf = Vec::new();
        let images = {
            icon_file.read_to_end(&mut buf).ok()?;
            xparser::parse_xcursor(&buf)?
        };
*/

        Self {
            timer,
            wgpu_engine,
            screen_task,
            platform,
            ews,
            geometry_manager,
            messages
        }

    }

    pub fn run(&mut self,event_loop: &mut calloop::EventLoop<Self>){
        use std::os::unix::io::AsRawFd;

        let interest = calloop::Interest {
            readable: true,
            writable: false,
        };
        let loop_handle = event_loop.handle();
        let loop_signal = event_loop.get_signal();
        let platform_event_source = loop_handle.insert_source(calloop::generic::Generic::new(self.platform.as_raw_fd(),interest,calloop::Mode::Edge),move|_event,_metadata,_data|Ok(calloop::PostAction::Continue)).unwrap();
        let wayland_event_source = loop_handle.insert_source(calloop::generic::Generic::new(self.ews.as_raw_fd(),interest,calloop::Mode::Edge),move|_event,_metadata,_data|Ok(calloop::PostAction::Continue)).unwrap();
        //let (waker,waker_source) = calloop::ping::make_ping().unwrap();
        //loop_handle.insert_source(waker_source, |_event,_metadata,_data|{}).unwrap();

        event_loop.run(None,self,|wcomp|{
            let redraw = wcomp.process_messages();
            if redraw {
                wcomp.wgpu_engine.dispatch_tasks();
                let time = wcomp.timer.elapsed().as_millis() as u32;
                let mut to_be_removed = Vec::new();
                wcomp.geometry_manager.surfaces_ref().for_each(|surface|{
                    let result = ews::with_states(surface.handle(),|surface_data|{
                        surface_data.cached_state.current::<ews::SurfaceAttributes>().frame_callbacks.drain(..).for_each(|callback|callback.done(time));
                    });
                    match result {
                        Ok(_)=>(),
                        Err(_)=>to_be_removed.push(surface.id)
                    }
                });

                to_be_removed.iter().for_each(|id|{
                    wcomp.wgpu_engine.task_handle_cast_mut(&wcomp.screen_task, |screen_task: &mut ScreenTask|{
                        screen_task.remove_surface(*id);
                    });
                    wcomp.geometry_manager.del_surface(*id);
                });

                if to_be_removed.is_empty() {
                    //waker.ping();
                    loop_signal.wakeup();
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(1000/60));
        }).unwrap();

        loop_handle.remove(platform_event_source);
        loop_handle.remove(wayland_event_source);
    }
}


