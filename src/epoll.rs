use std::io::Error;

use anyhow::{Result, bail};
use libc::{EPOLL_CTL_ADD, EPOLL_CTL_DEL, c_int, epoll_ctl, epoll_event};

pub fn register_interest(epoll_fd: i32, concerned_fd: i32, event: &mut epoll_event) -> Result<()> {
    if unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, concerned_fd, event) } < 0 {
        bail!(
            "could not register interest in events associated with FD: {concerned_fd}, error: {}",
            Error::last_os_error()
        );
    } else {
        Ok(())
    }
}

pub fn unregister_interest(epoll_fd: i32, concerned_fd: i32, event: &mut epoll_event) -> Result<()> {
    if unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_DEL, concerned_fd, event) } < 0 {
        bail!(
            "could not unregister interest in events associated with FD: {concerned_fd}, error: {}",
            Error::last_os_error()
        );
    } else {
        Ok(())
    }
}

pub fn new_epoll_event(events_interested_in: i32, identifier: i32) -> epoll_event {
    epoll_event {
        events: events_interested_in as u32,
        u64: identifier as u64,
    }
}


pub fn has_flag(flags_bitmask: u32, flag_to_check: c_int) -> bool {
    flags_bitmask & flag_to_check as u32 != 0
}