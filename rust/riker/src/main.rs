use riker::actors::*;

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

struct Link {
    div: Option<Int>,
    next: ActorRef<Int>,
}

impl ActorFactoryArgs<ActorRef<Int>> for Link {
    fn create_args(next: ActorRef<Int>) -> Self {
        Link { div: None, next }
    }
}

impl Actor for Link {
    type Msg = Int;
    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Int, _: Sender) {
        match self.div {
            None => {
                self.next.tell(msg, None);
                self.div = Some(msg);
                self.next = ctx
                    .actor_of_args::<Link, _>(&format!("{}", msg), self.next.clone())
                    .unwrap();
            }
            Some(i) => {
                if !msg.multiple_of(&i) {
                    self.next.tell(msg, None);
                }
            }
        }
    }
}

#[derive(Default)]
struct Printer;

impl Actor for Printer {
    type Msg = Int;
    fn recv(&mut self, _: &Context<Self::Msg>, msg: Int, _: Sender) {
        println!("{:?}", msg);
    }
}

struct Generator {
    sink: ActorRef<Int>,
    cur: Int,
    max: Int,
}

impl ActorFactoryArgs<(ActorRef<Int>, Int, Int)> for Generator {
    fn create_args((sink, cur, max): (ActorRef<Int>, Int, Int)) -> Self {
        Generator { sink, cur, max }
    }
}
#[derive(Debug, Clone)]
struct Emit;

impl Actor for Generator {
    type Msg = Emit;
    fn post_start(&mut self, ctx: &Context<Self::Msg>) {
        ctx.myself.tell(Emit, None);
    }
    fn recv(&mut self, ctx: &Context<Self::Msg>, _: Self::Msg, _: Sender) {
        if self.cur <= self.max {
            self.sink.tell(self.cur, None);
            self.cur.increment();
            ctx.myself.tell(Emit, None);
        }
    }
}

fn main() {
    let system = ActorSystem::new().unwrap();
    let tail = system
        .actor_of_args::<Link, _>("tail", system.actor_of::<Printer>("printer").unwrap())
        .unwrap();
    let _gen = system
        .actor_of_args::<Generator, _>("generator", (tail, Int(2), Int(1000)))
        .unwrap();
}
