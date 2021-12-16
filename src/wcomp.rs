use crate::geometry_manager::*;
use pal::PlatformBackend;
use screen_task::ScreenTask;
use std::cell::RefCell;
use std::rc::Rc;
use wgpu_engine::*;

pub struct WComp {
    pub(crate) timer: std::time::Instant,
    pub(crate) redraw_timer: std::time::Instant,
    pub(crate) fps: usize,
    pub(crate) wgpu_engine: wgpu_engine::WGpuEngine,
    pub(crate) screen_task: TaskId,
    pub(crate) platform: pal::Platform,
    pub(crate) ews: ews::EmbeddedWaylandServer,
    pub(crate) geometry_manager: GeometryManager,
    pub(crate) async_requests: Rc<RefCell<Vec<WCompRequest>>>,
    //pub(crate) default_cursor: usize
}
impl WComp {
    pub fn new() -> Self {
        let features_and_limits = ScreenTask::features_and_limits();

        let mut wgpu_engine = wgpu_engine::WGpuEngine::new(features_and_limits.clone())
            .expect("Failed to intialize the graphic engine");
        let screen_task = wgpu_engine
            .create_task(
                "ScreenTask".into(),
                features_and_limits,
                |_id, _tokio_runtime, update_context| ScreenTask::new(update_context),
            )
            .unwrap();

        let mut platform = pal::Platform::new(vec![Box::new(wgpu_engine.wgpu_context())]);
        if platform.platform_type() == pal::PlatformType::Compositor {
            platform.request(vec![
                pal::Request::Surface {
                    request: pal::SurfaceRequest::Create(None),
                },
                pal::Request::Surface {
                    request: pal::SurfaceRequest::Create(None),
                },
            ]);
        }

        let parameters = ews::Parameters {
            shm_formats: Vec::new(),
            drm_formats: vec![ews::DrmFormat {
                code: ews::DrmFourcc::Xrgb8888,
                modifier: ews::DrmModifier::Linear,
            }],
        };
        let ews = ews::EmbeddedWaylandServer::new(parameters);
        //ews.set_shm_formats(vec![Format::Argb8888,Format::Xrgb8888]);

        let geometry_manager = GeometryManager::new();
        let timer = std::time::Instant::now();
        let redraw_timer = std::time::Instant::now();
        let fps = 60;
        let async_requests = Rc::new(RefCell::new(Vec::new()));

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
            redraw_timer,
            fps,
            wgpu_engine,
            screen_task,
            platform,
            ews,
            geometry_manager,
            async_requests,
        }
    }

    pub fn run(&mut self, event_loop: &mut calloop::EventLoop<Self>) {
        use std::os::unix::io::AsRawFd;

        let interest = calloop::Interest {
            readable: true,
            writable: false,
        };
        let loop_handle = event_loop.handle();
        let platform_event_source = loop_handle
            .insert_source(
                calloop::generic::Generic::new(
                    self.platform.as_raw_fd(),
                    interest,
                    calloop::Mode::Edge,
                ),
                move |_event, _metadata, _data| Ok(calloop::PostAction::Continue),
            )
            .unwrap();
        let wayland_event_source = loop_handle
            .insert_source(
                calloop::generic::Generic::new(self.ews.as_raw_fd(), interest, calloop::Mode::Edge),
                move |_event, _metadata, _data| Ok(calloop::PostAction::Continue),
            )
            .unwrap();

        let timer = calloop::timer::Timer::new().unwrap();
        let timer_handle = timer.handle();
        let timer_event_source = loop_handle
            .insert_source(timer, move |_event: (), _metadata, _data| ())
            .unwrap();

        let mut redraw = false;
        event_loop
            .run(None, self, |wcomp| {
                redraw |= wcomp.process_messages();
                if redraw {
                    let current_time = wcomp.redraw_timer.elapsed();
                    let redraw_duration = std::time::Duration::from_millis(1000 / wcomp.fps as u64);
                    if current_time >= redraw_duration {
                        wcomp.wgpu_engine.dispatch_tasks();

                        let time = wcomp.timer.elapsed().as_millis() as u32;
                        let mut to_be_removed = Vec::new();
                        wcomp.geometry_manager.surfaces_ref().for_each(|surface| {
                            surface.handle().map(|handle| {
                                let result = ews::with_states(handle, |surface_data| {
                                    let mut attributes = surface_data
                                        .cached_state
                                        .current::<ews::SurfaceAttributes>();
                                    attributes
                                        .frame_callbacks
                                        .drain(..)
                                        .for_each(|callback| callback.done(time));
                                });
                                match result {
                                    Ok(_) => (),
                                    Err(_) => to_be_removed.push(surface.id()),
                                }
                            });
                        });
                        wcomp.redraw_timer = std::time::Instant::now();
                        redraw = false;
                    } else {
                        timer_handle.add_timeout(redraw_duration - current_time, ());
                    }
                }
            })
            .unwrap();

        loop_handle.remove(platform_event_source);
        loop_handle.remove(wayland_event_source);
        loop_handle.remove(timer_event_source);
    }
}
