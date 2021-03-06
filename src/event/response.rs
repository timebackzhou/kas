// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Event handling: Response type

use super::{Action, Event};

/// Response type from [`Handler::handle`].
///
/// This type wraps [`Handler::Msg`] allowing both custom messages and toolkit
/// messages.
///
/// [`Handler::handle`]: super::Handler::handle
/// [`Handler::Msg`]: super::Handler::Msg
#[derive(Clone, Debug)]
#[must_use]
pub enum Response<M> {
    /// No action
    None,
    /// Unhandled input events get returned back up the widget tree
    Unhandled(Event),
    /// Custom message type
    Msg(M),
}

// Unfortunately we cannot write generic `From` / `TryFrom` impls
// due to trait coherence rules, so we impl `from` etc. directly.
impl<M> Response<M> {
    /// True if variant is `None`
    #[inline]
    pub fn is_none(&self) -> bool {
        match self {
            &Response::None => true,
            _ => false,
        }
    }

    /// Produce [`Response::Unhandled`] variant from an [`Action`]
    ///
    /// Convenience function for common usage.
    #[inline]
    pub fn unhandled_action(action: Action) -> Self {
        Response::Unhandled(Event::Action(action))
    }

    /// Map from one `Response` type to another
    ///
    /// Once Rust supports specialisation, this will likely be replaced with a
    /// `From` implementation.
    #[inline]
    pub fn from<N>(r: Response<N>) -> Self
    where
        M: From<N>,
    {
        r.try_into()
            .unwrap_or_else(|msg| Response::Msg(M::from(msg)))
    }

    /// Map one `Response` type into another
    ///
    /// Once Rust supports specialisation, this will likely be redundant.
    #[inline]
    pub fn into<N>(self) -> Response<N>
    where
        N: From<M>,
    {
        Response::from(self)
    }

    /// Try mapping from one `Response` type to another, failing on `Msg`
    /// variant and returning the payload.
    #[inline]
    pub fn try_from<N>(r: Response<N>) -> Result<Self, N> {
        use Response::*;
        match r {
            None => Ok(None),
            Unhandled(e) => Ok(Unhandled(e)),
            Msg(m) => Err(m),
        }
    }

    /// Try mapping one `Response` type into another, failing on `Msg`
    /// variant and returning the payload.
    #[inline]
    pub fn try_into<N>(self) -> Result<Response<N>, M> {
        Response::try_from(self)
    }
}

impl<M> From<M> for Response<M> {
    fn from(msg: M) -> Self {
        Response::Msg(msg)
    }
}
