/**
 * (c) Incomplete Worlds 2020 
 * Alberto Fernandez (ajfg)
 *
 * Redirect an HTTP request to another URL 
 */

//use std::cell::RefCell;
use std::pin::Pin;
//use std::rc::Rc;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, Error, HttpResponse};

use futures::future::{ok, /*Either,*/ Ready};
use futures::Future;




pub struct RedirectRequest;

impl<S, B> Transform<S> for RedirectRequest
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RedirectRequestMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RedirectRequestMiddleware { service })
        // ok(RedirectRequestMiddleware {
        //     service: Rc::new(RefCell::new(service)),
        // })
    }
}
pub struct RedirectRequestMiddleware<S> {
    service: S,
    //service: Rc<RefCell<S>>,
}

impl<S, B> Service for RedirectRequestMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if req.path().contains("/mcsaas") == true {
            Box::pin(async move {
                // Redirect to the MCS
                Ok(req.into_response(
                    HttpResponse::Found()
                        .header(http::header::LOCATION, "/mcsaas")
                        .finish()
                        .into_body(),
                    )
                )
            })
        } else if req.path().contains("/fdsaas") == true {
            // Redirect to the FDS
            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Found()
                        .header(http::header::LOCATION, "/fdsaas")
                        .finish()
                        .into_body(),
                   )
                )
            })
        } else {
            // Call the service as normal
            let fut = self.service.call(req);
            
            Box::pin(async move {
                 let res = fut.await?;

                 Ok(res)
            })
        }
    }
}