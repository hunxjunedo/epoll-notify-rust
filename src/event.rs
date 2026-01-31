

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub(crate) events: u32, // a bitmask of the events that took place OR info that you want to pass to the OS
    pub(crate) epoll_data: usize, // a unique id you associate with this event. So once you get a notification, you can map it and know that this particular action triggered the notification. can even be a pointer to maybe some data
}

impl Default for Event {
    fn default() -> Self {
        Self {
            events: 0,
            epoll_data: 0,
        }
    }
}
