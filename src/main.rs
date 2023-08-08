use actix_web::{web::JsonConfig, App, HttpRequest, HttpResponse, HttpServer, Result};
use dotenv::dotenv;
use paperclip::actix::{
    api_v2_operation,
    web::{self},
    Apiv2Schema, OpenApiExt,
};
use paperclip::v2::models::DefaultApiRaw;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::fs::read;
use tokio::sync::{mpsc, oneshot};
use web_push::{
    ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

/// This struct is used to hold the state of the application.
///
/// It contains the sender part of an asynchronous channel.
/// Messages sent through this channel will contain requests for push notifications.
struct AppState {
    push_queue: mpsc::Sender<PushaRequest>,
}

/// This struct represents the search result read from the database.
///
/// It contains the ID and distance of the search result.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Apiv2Schema)]
pub struct ReadSearchResult {
    pub id: usize,
    pub distance: f32,
}

/// This struct represents the request to generate embeddings.
///
/// It contains a vector of sentences for which embeddings need to be generated.
#[derive(Deserialize, Serialize, Apiv2Schema)]
struct EmbeddingRequest {
    sentences: Vec<String>,
}

/// This struct represents the request to send push notifications.
///
/// It contains the endpoint, expiration time and keys required to send a push notification.
#[derive(Deserialize, Serialize, Apiv2Schema)]
pub struct PushRequest {
    endpoint: String,
    expiration_time: Option<String>,
    keys: Keys,
    sentence: String,
}

/// This struct holds keys necessary to send a push notification.
#[derive(Deserialize, Serialize, Apiv2Schema)]
pub struct Keys {
    p256dh: String,
    auth: String,
}

/// This struct is used to send a request for a push notification.
///
/// It contains the sentences to be used, a sender for the response, and information about the subscription.
#[derive(Debug)]
struct PushaRequest {
    sentences: Vec<String>,
    responder: oneshot::Sender<Vec<f32>>,
    subscription_info: SubscriptionInfo,
}

/// This struct contains the methods used to send push notifications.

struct Pusha {}

impl Pusha {
    fn start_worker(mut receiver: mpsc::Receiver<PushaRequest>) {
        tokio::spawn(async move {
            while let Some(PushaRequest {
                sentences,
                responder,
                subscription_info,
            }) = receiver.recv().await
            {
                tracing::info!("worker: {:?}", sentences);

                let ttl = None;
                let push_payload = Some(sentences.join(" "));

                let ece_scheme = ContentEncoding::Aes128Gcm;

                let subscription_info: SubscriptionInfo = SubscriptionInfo::new(
                    &subscription_info.endpoint,
                    &subscription_info.keys.p256dh,
                    &subscription_info.keys.auth,
                );

                let mut builder = WebPushMessageBuilder::new(&subscription_info).unwrap();

                if let Some(ref payload) = push_payload {
                    builder.set_payload(ece_scheme, payload.as_bytes());
                }

                if let Some(time) = ttl {
                    builder.set_ttl(time);
                }

                let mut sig_builder = VapidSignatureBuilder::from_pem(
                    env::var("VAPID_PRIVATE_KEY").unwrap().as_bytes(),
                    &subscription_info,
                )
                .unwrap();

                sig_builder.add_claim("sub", "mailto:test@example.com");
                sig_builder.add_claim("foo", "bar");
                sig_builder.add_claim("omg", 123);

                let signature = sig_builder.build().unwrap();

                builder.set_vapid_signature(signature);

                let client = WebPushClient::new().unwrap();

                client.send(builder.build().unwrap()).await.unwrap();

                let result = vec![1.0];
                let _ = responder.send(result);
                tracing::info!("worker: done")
            }
        });
    }
}

/// The `push` function is used to send a push notification.
///
/// It accepts the state of the application and the request body which contains necessary information to send the notification.
/// It returns a result, which if successful, contains a JSON response, otherwise contains an error.
#[api_v2_operation(
    summary = "Send a push notification",
    description = "This endpoint sends a push notification using the provided parameters"
)]
async fn push(
    app_state: web::Data<AppState>,
    req_body: web::Json<PushRequest>,
) -> Result<web::Json<String>, actix_web::Error> {
    tracing::info!("push: {:?}", req_body.endpoint);

    let (responder, _receiver) = oneshot::channel();
    let subscription_info: SubscriptionInfo = SubscriptionInfo::new(
        &req_body.endpoint,
        &req_body.keys.p256dh,
        &req_body.keys.auth,
    );

    let request = PushaRequest {
        sentences: vec![req_body.sentence.clone()],
        responder,
        subscription_info,
    };
    app_state.push_queue.send(request).await.unwrap();
    Ok(web::Json("queued".to_string()))
}

/// The `index` function serves the index page.
///
/// It accepts the HttpRequest as an argument, then retrieves and serves the requested file.
/// If the path is a directory, the index.html file in that directory will be served.
#[api_v2_operation(
    summary = "Index page"
    description = "This endpoint serves the index page"
)]
async fn index(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    // if path is a directory, add index.html
    let path = if path == std::path::PathBuf::from("") {
        path.join("index.html")
    } else {
        path
    };
    let data = read(format!("./web/{}", path.to_str().unwrap())).await?;
    // add mime type
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Ok(HttpResponse::Ok().content_type(mime.to_string()).body(data))
}

/// The `main` function sets up the environment and starts the HTTP server.
///
/// It sets up logging with `tracing`, creates an instance of `AppState`, sets up the HTTP server, and starts it.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Configure and set tracing subscriber
    let subscriber = tracing_subscriber::FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");

    let host = env::var("HOST").unwrap_or_else(|_| "[::0]:8081".to_string());
    tracing::info!("Starting web server... {}", host);

    let (sender, receiver) = mpsc::channel(100);
    Pusha::start_worker(receiver);
    let app_state = actix_web::web::Data::new(AppState { push_queue: sender });

    // set app level spec
    let mut spec = DefaultApiRaw::default();
    spec.info.version = "0.0.2".into();
    spec.info.title = "king-pusha".into();
    tracing::info!("Starting web server...");

    let server = HttpServer::new(move || {
        App::new()
            .wrap_api_with_spec(spec.clone())
            .app_data(JsonConfig::default().limit(2 * 1024 * 1024))
            .app_data(app_state.clone())
            .service(web::resource("/push").route(web::post().to(push)))
            .with_json_spec_at("/api/spec/v2") // more specific than catch-all route
            .service(web::resource("/{filename:.*}").route(web::get().to(index)))
            .build()
    })
    .bind(host)?
    .run();

    // run server
    server.await?;

    Ok(())
}
