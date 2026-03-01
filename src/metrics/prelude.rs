use actix_web::{web::Data, Error};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::{ok, Ready};
use prometheus::{HistogramVec, HistogramOpts, IntCounterVec, Opts, Registry};
use std::task::{Context, Poll};
use std::time::Instant;
use std::pin::Pin;

#[derive(Clone)]
pub struct MetricsCollector {
    request_counter: IntCounterVec,
    response_time_histogram: HistogramVec,
    request_size: IntCounterVec,
}

impl MetricsCollector {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let request_counter = IntCounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "endpoint", "status"]
        )?;
        
        let response_time_histogram = HistogramVec::new(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds"),
            &["method", "endpoint"]
        )?;
        
        let request_size = IntCounterVec::new(
            Opts::new("http_request_size_bytes", "HTTP request size in bytes"),
            &["method", "endpoint"]
        )?;
        
        registry.register(Box::new(request_counter.clone()))?;
        registry.register(Box::new(response_time_histogram.clone()))?;
        registry.register(Box::new(request_size.clone()))?;

        Ok(MetricsCollector {
            request_counter,
            response_time_histogram,
            request_size,
        })
    }

    fn get_endpoint_pattern(req: &ServiceRequest) -> String {
        req.match_pattern()
            .unwrap_or_else(|| req.path().to_string())
    }
}

/// Honest reaction:
/// https://upload.wikimedia.org/wiktionary/en/f/f1/Soyjak.jpg
///
/// This is the wrapper that we use... gets called for all the
/// endpoints. WITHOUT NEEDING TO CHANGE ANY OF THE ENDPOINT
/// SOURCE CODE.
pub struct MetricsMiddleware {
    collector: Data<MetricsCollector>,
}

impl MetricsMiddleware {
    pub fn new(collector: Data<MetricsCollector>) -> Self {
        MetricsMiddleware { collector }
    }
}

/// This is the actual interface that actix web uses to autonatically
/// run our "metrics code" during the lifecycle of our services (endpoints).
///
/// Docs: https://docs.rs/actix-web/latest/actix_web/dev/trait.Transform.html
///
/// The point here is to make a callback to our "MetricsMiddlewareService" so
/// that IT can register metrics of our "service".
///
/// So here we simply want to capture and send service information further
/// down below. (A "factory")
impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddlewareService {
            service,
            collector: self.collector.clone(),
        })
    }
}

/// This is where the "fun" happens.
pub struct MetricsMiddlewareService<S> {
    service: S,
    collector: Data<MetricsCollector>,
}


/// To actually be able to interact with services asynchronously
/// we need to implement this actix_web trait:
///
/// Docs: https://docs.rs/actix-web/latest/actix_web/dev/trait.Service.html
///
/// This is here to allow us to interface with the actix web scheduler (or
/// whatever they would call it) running somewhere beneath our code.
impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let collector = self.collector.clone();
        
        let method: String = req.method().to_string();
        let endpoint = MetricsCollector::get_endpoint_pattern(&req).to_string();
        
        if let Some(content_length) = req.headers().get("content-length")
        && let Ok(size_str) = content_length.to_str()
        && let Ok(size) = size_str.parse::<u64>() {
            collector.request_size
                .with_label_values(&[&method, &endpoint])
                .inc_by(size);
        }

        // Here is where EVERYTHING happens. We simply call
        // the unerlying service and let it do... whatever.
        //
        // Here `self.service` may be api::server::create_server
        // or any other service. It doesn't matter we simply
        // create a "future" object working as a callback
        // to the service. So we can interact with it whenever
        // it's done processing stuff. (including dialing
        // back to the user, and answering their request.)
        let fut = self.service.call(req);

        // This is a quirk of actix_web (or futures?): see above that
        // we want to return self::Future which is actually:
        // Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>
        //
        // I believe this might be needed to prevent some
        // memory mangling through pinning (or freezing)
        // memory in place.
        //
        // I could be wrong.
        Box::pin(async move {
            // And here is the callback! It's that simple. Whatever
            // else happens below is just simple metrics stuff
            // (difference between start time and now, incrementing
            // counter for requests total, etc.)
            let result = fut.await;
            let duration = start_time.elapsed().as_secs_f64();
            
            match result {
                Ok(res) => {
                    collector.response_time_histogram
                        .with_label_values(&[&method, &endpoint])
                        .observe(duration);
                    
                    let status = res.status().as_u16().to_string();
                    collector.request_counter
                        .with_label_values(&[&method, &endpoint, &status])
                        .inc();

                    Ok(res)
                },
                Err(e) => {
                    collector.response_time_histogram
                        .with_label_values(&[&method, &endpoint])
                        .observe(duration);
                    
                    collector.request_counter
                        .with_label_values(&[&method, &endpoint, &"error".to_string()])
                        .inc();
                    
                    Err(e)
                }
            }
        })
    }
}
