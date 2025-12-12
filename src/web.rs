use embassy_time::Timer;
use picoserve::{
    AppBuilder, Router,
    response::{self, File, IntoResponse},
    routing,
};

pub struct Application;

impl AppBuilder for Application {
    type PathRouter = impl routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        Router::new()
            .route(
                "/",
                routing::get(async || {
                    response::Response::empty(response::StatusCode::new(302))
                        .with_header("location", "http://192.168.2.1:80/wow")
                }),
            )
            .route(
                "/wow",
                routing::get_service(File::html(include_str!("index.html"))),
            )
            .route(
                "/form",
                routing::get_service(File::html(include_str!("form.html"))),
            )
            .route(
                "/api/led",
                routing::post(handle_led_request),
                //         .get(|| async {
                //         if let Some(signal) = crate::led::LED_STATE.try_take() {
                //             return response::Json(signal);
                //         }
                //         defmt::error!("could not take LED_STATE");
                //         response::Json(crate::led::Leds::default())
                //     }),
            )
            .route("/api/jsonshema", routing::get(get_plain_shema))
    }
}

async fn get_plain_shema() -> impl IntoResponse {
    response::Json(crate::led::Zones::default())
}

async fn handle_led_request(
    input: picoserve::extract::Json<crate::led::Zones>,
) -> impl IntoResponse {
    defmt::info!("led post request json data to led [{:?}]", input.0);
    for _i in 0..1 {
        crate::led::LED_STATE.signal(input.0);
        Timer::after_millis(75).await;
    }
    response::StatusCode::OK
}

pub const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    router: &'static picoserve::AppRouter<Application>,
    config: &'static picoserve::Config<embassy_time::Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::Server::new(router, config, &mut http_buffer)
        .listen_and_serve(
            task_id,
            stack,
            port,
            &mut tcp_rx_buffer,
            &mut tcp_tx_buffer,
        )
        .await
        .into_never()
}

pub struct WebApp {
    pub router: &'static Router<<Application as AppBuilder>::PathRouter>,
    pub config: &'static picoserve::Config<embassy_time::Duration>,
}

impl Default for WebApp {
    fn default() -> Self {
        let router = picoserve::make_static!(
            picoserve::AppRouter<Application>,
            Application.build_app()
        );

        let config = picoserve::make_static!(
            picoserve::Config<embassy_time::Duration>,
            picoserve::Config::new(picoserve::Timeouts {
                start_read_request: Some(embassy_time::Duration::from_secs(5)),
                persistent_start_read_request: Some(
                    embassy_time::Duration::from_secs(1)
                ),
                read_request: Some(embassy_time::Duration::from_secs(1)),
                write: Some(embassy_time::Duration::from_secs(1)),
            })
            .keep_connection_alive()
        );

        Self { router, config }
    }
}
