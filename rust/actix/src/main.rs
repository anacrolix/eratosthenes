use actix::prelude::*;

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

impl Message for Int {
    type Result = ();
}

struct Link {
    div: Option<Int>,
    next: Recipient<Int>,
}

impl Actor for Link {
    type Context = Context<Self>;
}

impl Handler<Int> for Link {
    type Result = ();
    fn handle(&mut self, msg: Int, _: &mut Self::Context) {
        match self.div {
            None => {
                self.next.try_send(msg).unwrap();
                self.div = Some(msg);
                self.next = Link {
                    div: None,
                    next: self.next.clone(),
                }
                .start()
                .recipient();
            }
            Some(i) => {
                if !msg.multiple_of(&i) {
                    self.next.do_send(msg).unwrap();
                }
            }
        }
    }
}

#[derive(Default)]
struct Printer;

impl Actor for Printer {
    type Context = Context<Self>;
}

impl Handler<Int> for Printer {
    type Result = ();
    fn handle(&mut self, msg: Int, _: &mut Self::Context) {
        println!("{:?}", msg);
    }
}

struct Generator {
    sink: Recipient<Int>,
    cur: Int,
    max: Int,
}

impl Actor for Generator {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify(Emit);
    }
}

struct Emit;
impl Message for Emit {
    type Result = ();
}

impl Handler<Emit> for Generator {
    type Result = ();
    fn handle(&mut self, _: Emit, ctx: &mut Self::Context) {
        if self.cur <= self.max {
            self.sink.do_send(self.cur).unwrap();
            self.cur.increment();
            ctx.notify(Emit);
        }
    }
}

fn main() {
    let system = System::new("test");
    let chain = Link {
        div: None,
        next: Printer::start_default().recipient(),
    }
    .start()
    .recipient();
    Generator {
        sink: chain,
        cur: Int(2),
        max: Int(100000),
    }
    .start();
    system.run().unwrap();
}
