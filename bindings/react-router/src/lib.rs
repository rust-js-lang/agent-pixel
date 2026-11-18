//! Rust bindings for React Router's declarative browser navigation.
use react::{JSX, jsx};

pub struct BrowserRouterProps {
    pub children: JSX::Element,
}

#[cfg_attr(rust_js, rust_js::link_name = "react-router#BrowserRouter")]
fn Router(props: BrowserRouterProps) -> JSX::Element {
    unreachable!()
}

pub fn BrowserRouter(BrowserRouterProps { children }: BrowserRouterProps) -> JSX::Element {
    jsx! { <Router>{children}</Router> }
}

pub struct Location {
    pub pathname: String,
    pub search: String,
}

/// The string overload of NavigateFunction in a declarative router.
pub type Navigate = dyn Fn(&str);

#[cfg_attr(rust_js, rust_js::link_name = "react-router#useLocation")]
fn use_location() -> &'static Location {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "react-router#useNavigate")]
fn use_navigate() -> &'static Navigate {
    unreachable!()
}

pub struct Navigation {
    pub location: &'static Location,
    pub navigate: &'static Navigate,
}

pub fn use_browser_navigation() -> Navigation {
    Navigation {
        location: use_location(),
        navigate: use_navigate(),
    }
}
