use x11_dl::xlib;

type MockHandle = i32;

#[derive(Debug, Clone, PartialEq)]
pub enum WindowHandle {
    MockHandle(MockHandle),
    XlibHandle(xlib::Window),
}

#[derive(Debug, Clone)]
pub struct Window {
    pub handle: WindowHandle,
    pub visable: bool,
    pub floating: bool,
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub border: i32,
    pub margin: i32,
    height: i32,
    width: i32,
    x: i32,
    y: i32,
}

impl Window {
    pub fn new(h: WindowHandle, name: Option<String>) -> Window {
        Window {
            handle: h,
            visable: false,
            floating: false,
            name,
            tags: Vec::new(),
            border: 1,
            margin: 10,
            height: 600,
            width: 800,
            x: 0,
            y: 0,
        }
    }

    pub fn set_height(&mut self, height: i32) {
        self.height = height
    }
    pub fn set_width(&mut self, width: i32) {
        self.width = width
    }
    pub fn set_x(&mut self, x: i32) {
        self.x = x 
    }
    pub fn set_y(&mut self, y: i32) {
        self.y = y
    }

    pub fn height(&self) -> i32 {
        self.height 
            - (self.margin * 2)
            - (self.border * 2)
    }
    pub fn width(&self) -> i32 {
        self.width 
            - (self.margin * 2)
            - (self.border * 2)
    }
    pub fn x(&self) -> i32 {
        self.x + self.margin
    }
    pub fn y(&self) -> i32 {
        self.y + self.margin
    }

    pub fn tag(&mut self, tag: String) {
        if tag == "" {
            return;
        }
        for t in &self.tags {
            if t == &tag {
                return;
            }
        }
        self.tags.push(tag);
    }

    pub fn clear_tags(&mut self) {
        self.tags = vec![];
    }

    pub fn has_tag(&self, tag: String) -> bool {
        self.tags.contains(&tag)
    }

    pub fn untag(&mut self, tag: String) {
        let mut new_tags: Vec<String> = Vec::new();
        for t in &self.tags {
            if t != &tag {
                new_tags.push(t.clone())
            }
        }
        self.tags = new_tags;
    }
}

#[test]
fn should_be_able_to_tag_a_window() {
    let mut subject = Window::new(WindowHandle::MockHandle(1), None);
    subject.tag("test".to_string());
    assert!(
        subject.has_tag("test".to_string()),
        "was unable to tag the window"
    );
}

#[test]
fn should_be_able_to_untag_a_window() {
    let mut subject = Window::new(WindowHandle::MockHandle(1), None);
    subject.tag("test".to_string());
    subject.untag("test".to_string());
    assert!(
        subject.has_tag("test".to_string()) == false,
        "was unable to untag the window"
    );
}
