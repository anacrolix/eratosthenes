use stakker::*;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
struct Int(u64);

impl Int {
    fn multiple_of(&self, div: &Int) -> bool {
        self.0 % div.0 == 0
    }
    fn increment(&mut self) {
        self.0 += 1
    }
}

impl std::fmt::Display for Int {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.0.fmt(fmt)
    }
}

type LinkNext = ActorOwn<Link>;

struct Link {
    div: Option<Int>,
    next: LinkNext,
}

impl Link {
    fn init_tail(_: CX![], next: LinkNext) -> Option<Self> {
        Some(Link { div: None, next })
    }
    fn init_div(div: Int, next: LinkNext) -> Option<Self> {
        Some(Link {
            div: Some(div),
            next,
        })
    }
}

trait Next {
    fn recv(&mut self, _: CX![], _: Int)
    where
        Self: std::marker::Sized;
}

impl Next for Link {
    fn recv(&mut self, cx: CX![], msg: Int) {
        match self.div {
            None => {
                call!([self.next], recv(msg));
                self.div = Some(msg);
                self.next = actor!(cx, Link::init_tail(self.next), ret_nop!());
                // eprintln!("created {:?}", self.next.path());
            }
            Some(i) => {
                if !msg.multiple_of(&i) {
                    call!([self.next], recv(msg));
                }
            }
        }
    }
}

#[derive(Default)]
struct Printer;

impl Printer {
    fn init(_: CX![]) -> Option<Self> {
        Some(Self)
    }
    fn recv(&mut self, _: CX![], msg: Int) {
        println!("{}", msg);
    }
}

struct Generator {
    sink: LinkNext,
    cur: Int,
    max: Option<Int>,
}

impl Generator {
    fn init(cx: CX![], sink: LinkNext) -> Option<Self> {
        call!([cx], emit());
        Some(Self {
            sink,
            cur: Int(2),
            max: None,
        })
    }
    fn emit(&mut self, cx: CX![]) {
        if self.max.map_or(true, |max| self.cur <= max) {
            call!([self.sink], recv(self.cur));
            self.cur.increment();
            call!([cx], emit());
        }
    }
}

fn main() {
    use std::time::{Duration, Instant};
    let mut system = Stakker::new(Instant::now());
    let printer = actor!(system, Printer::init(), ret_nop!());
    let tail = actor!(system, Link::init_tail(printer), ret_nop!());
    let _gen = actor!(system, Generator::init(tail), ret_nop!());
    let stakker = system;
    stakker.run(Instant::now(), false);
    while stakker.not_shutdown() {
        let maxdur = stakker.next_wait_max(Instant::now(), Duration::from_secs(60), false);
        std::thread::sleep(maxdur);
        stakker.run(Instant::now(), false);
    }
}
