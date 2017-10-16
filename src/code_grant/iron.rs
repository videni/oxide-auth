use super::*;
use iron;
use iron::modifiers::Redirect;
use iron::IronResult;
use iron::Response;
use iron::Request as IRequest;

pub struct IronGranter<A: Authorizer + Send + 'static> {
    authorizer: std::sync::Arc<std::sync::Mutex<std::cell::RefCell<A>>>
}

pub struct IronAuthorizer<A: Authorizer + Send + 'static> {
    authorizer: std::sync::Arc<std::sync::Mutex<std::cell::RefCell<A>>>
}

impl<A: Authorizer + Send + 'static> IronGranter<A> {
    pub fn new(data: A) -> IronGranter<A> {
        IronGranter { authorizer: std::sync::Arc::new(std::sync::Mutex::new(std::cell::RefCell::new(data))) }
    }

    pub fn authorize(&self) -> IronAuthorizer<A> {
        IronAuthorizer { authorizer: self.authorizer.clone() }
    }
}

impl<'a, 'b> WebRequest for IRequest<'a, 'b> {
    fn owner_id(&self) -> Option<String> {
        return Some("test".to_string());
    }
}

impl<A: Authorizer + Send + 'static> iron::Handler for IronAuthorizer<A> {
    fn handle<'a>(&'a self, req: &mut iron::Request) -> IronResult<Response> {
        use std::ops::Deref;
        use std::ops::DerefMut;
        let urldecoded = match decode_query(&req.url) {
            Err(st) => return Ok(Response::with((iron::status::BadRequest, st))),
            Ok(res) => res
        };

        let locked = self.authorizer.lock().unwrap();
        let mut auth_ref = locked.deref().borrow_mut();
        let mut granter = IronGrantRef{0: auth_ref.deref_mut()};

        let negotiated = match granter.negotiate(urldecoded) {
           Err(st) => return Ok(Response::with((iron::status::BadRequest, st))),
           Ok(v) => v
        };

        let redirect_to = granter.authorize(
           req.owner_id().unwrap().into(),
           negotiated);
        Ok(Response::with((iron::status::Found, Redirect(redirect_to))))
    }
}
