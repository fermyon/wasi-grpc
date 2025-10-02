pub mod route_guide {
    tonic::include_proto!("routeguide");
}

use route_guide::route_guide_server::RouteGuide;
use route_guide::route_guide_server::RouteGuideServer;

use futures::Stream;
use tonic::{Request, Response, Status, Streaming};

#[wasi_grpc_server::grpc_component(RouteGuideServer)]
struct Svc;

#[tonic::async_trait]
impl RouteGuide for Svc {
    async fn get_feature(&self, request: Request<route_guide::Point>) -> Result<Response<route_guide::Feature>, Status> {
        let lat = request.get_ref().latitude;

        let name = if lat < 0 {
            "west"
        } else if lat > 0 {
            "east"
        } else {
            "on the meridian"
        };

        let feat = route_guide::Feature { name: name.into(), ..Default::default() };

        Ok(Response::new(feat))
    }

    type ListFeaturesStream = std::pin::Pin<Box<dyn futures::Stream<Item=Result<route_guide::Feature, Status>> + Send + 'static>>;

    async fn list_features(&self, _request: Request<route_guide::Rectangle>) -> Result<Response<Self::ListFeaturesStream>, Status> {
        let stm = async_stream::stream! {
            let pages = [
                "https://www.fermyon.com/blog/sqlite-is-edge-scale",
                "https://www.fermyon.com/blog/openapi-docs-for-spin-with-rust",
                "https://www.fermyon.com/blog/building-a-graphql-api-with-fwf"
            ];

            for page in pages {
                let req = spin_sdk::http::Request::get(page).build();
                let fut = spin_sdk::http::send::<spin_sdk::http::Request, spin_sdk::http::Response>(req);
                let resp = spin_executor::run(fut);

                yield match resp {
                    Ok(r) => Ok(route_guide::Feature { name: format!("{page} has content-len {:?}", r.header("content-length").and_then(|hv| hv.as_str())), location: None }),
                    Err(e) => Err(Status::from_error(Box::new(e)))
                }
            }
        };

        Ok(Response::new(Box::pin(stm)))
    }

    async fn record_route(&self, request: Request<Streaming<route_guide::Point>>) -> Result<Response<route_guide::RouteSummary>, Status> {
        let mut req = request;
        let r = req.get_mut();

        let mut distance = 0;
        let mut count = 0;
        let mut last_pt = None;

        loop {
            let Some(pt) = r.message().await? else {
                break;
            };

            count = count + 1;

            if let Some(last) = last_pt {
                distance = distance + dist(last, pt);
            }

            last_pt = Some(pt);
        }

        Ok(Response::new(route_guide::RouteSummary { point_count: count, feature_count: 0, distance, elapsed_time: 0 }))
    }

    type RouteChatStream = std::pin::Pin<Box<dyn Stream<Item = Result<route_guide::RouteNote, Status> > + Send + 'static>>;

    async fn route_chat(&self, request: Request<Streaming<route_guide::RouteNote>>) -> Result<Response<Self::RouteChatStream>, Status> {
        use futures::StreamExt;

        let stm = request.into_inner().flat_map(|m| {
            async_stream::stream! {
                match m {
                    Err(e) => { yield Err(e); }
                    Ok(m) => {
                        for i in 0..3 {
                            yield Ok(route_guide::RouteNote { message: format!("Note {i} for message {}", m.message), location: None });
                        }
                    }
                }
            }
        });

        Ok(tonic::Response::new(Box::pin(stm)))
    }
}

fn dist(pt1: route_guide::Point, pt2: route_guide::Point) -> i32 {
    let latd = pt1.latitude - pt2.latitude;
    let longd = pt1.longitude - pt2.longitude;
    let d = f64::sqrt((latd * latd + longd * longd) as f64);
    d as i32
}
