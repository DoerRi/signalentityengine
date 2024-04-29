/*
IDEA:
    Enitity comunicate via Signals

*/
use std::sync::mpsc::{self, *};
use std::thread::{self, *};

fn main() {
    println!("Hello, world!");
    let app = App::new(4);
}

//has a channel for comunication to the app and or to all threads
struct AppEntryPoint {}

struct App {
    joinhandler: Vec<JoinHandle<()>>,
    enititys: Vec<Entity>,
}
impl App {
    fn new(thread_count: u16) -> AppEntryPoint {
        //creating the channels
        let mut receivers: Vec<Receiver<Signal>> = vec![];
        let mut senders: Vec<Sender<Signal>> = vec![];
        for _i in 0..thread_count {
            let (send, rece): (Sender<Signal>, Receiver<Signal>) = mpsc::channel();
            receivers.push(rece);
            senders.push(send)
        }
        //createing the app and SignalThread objects
        let mut threads: Vec<SignalThread> = vec![];
        static mut APP: App = App {
            enititys: vec![],
            joinhandler: vec![],
        };
        for i in 0..thread_count {
            threads.push(SignalThread::new(
                unsafe { &mut APP },
                receivers.pop().unwrap(),
                senders.clone(),
                i,
            ));
        }
        //starting the threads
        for _i in 0..thread_count {
            let thisthread = threads.pop().unwrap(); //unsafe { app.threads.get(i as usize).unwrap() };
            unsafe {
                APP.joinhandler
                    .push(thread::spawn(move || thisthread.thread_fn()))
            };
        }

        AppEntryPoint {}
    }
}

struct SignalThread {
    id: u16,
    app: &'static mut App,
    srece: Receiver<Signal>,
    ssend: Vec<Sender<Signal>>,
}
impl SignalThread {
    fn new(
        app: &'static mut App,
        srece: Receiver<Signal>,
        ssend: Vec<Sender<Signal>>,
        id: u16,
    ) -> Self {
        SignalThread {
            app,
            srece,
            ssend,
            id,
        }
    }
    fn thread_fn(mut self) {
        loop {
            //fetch signal and test if it is valid
            let (send, rece): (Sender<Signal>, Receiver<Signal>) = mpsc::channel();
            let signal: Signal = self.srece.recv().unwrap();
            if signal.to as usize % self.app.joinhandler.len() != self.id as usize {
                continue;
            }
            //start signal handler of entity
            self.app
                .enititys
                .get(signal.to as usize)
                .unwrap()
                .handle_signal(signal, send);
            //sending generated signals
            rece.into_iter().for_each(|s| self.send_signal(s));
        }
    }
    fn send_signal(&mut self, signal: Signal) {
        let _ = self
            .ssend
            .get(signal.to as usize % self.app.joinhandler.len())
            .unwrap()
            .send(signal);
    }
}

unsafe impl Sync for SignalThread {}
unsafe impl Send for SignalThread {}



//Entity holds data and 
//  id = unic identifier & indicates which thread is responsible (thread_id = id % thread_count)
struct Entity {
    id: u64,
    enitity_data: Box<dyn EntityData>,
}
impl Entity {
    fn handle_signal(&self, signal: Signal, send_signal_queue: Sender<Signal>) {
        match signal.signaltype {
            //standart signals
            SignalType::Init => self.enitity_data.init(send_signal_queue),
            SignalType::Update(delta) => self.enitity_data.update(delta, send_signal_queue),
            SignalType::Delete() => {
                self.enitity_data.delete(send_signal_queue);
                self.delete();
            }
            //custom signals
            SignalType::CustomData(id, data) => {
                self.enitity_data.handle_signal(id, data, send_signal_queue)
            }
        }
    }
    fn delete(&self) {}
    fn add_new_enitiy(&self, enitity: &Entity) {}
}

trait EntityData {
    fn handle_signal(&self, id: u64, data: Box<dyn SignalData>, signal_queue: Sender<Signal>);
    fn update(&self, delta: f64, signal_queue: Sender<Signal>);
    fn init(&self, signal_queue: Sender<Signal>);
    fn delete(&self, signal_queue: Sender<Signal>);
}

struct Signal {
    to: u64,
    from: u64,
    signaltype: SignalType,
}
enum SignalType {
    Init,
    Update(f64),
    Delete(),
    CustomData(u64, Box<dyn SignalData>),
}

trait SignalData {
    fn get_data(&self) {}
}
