use crossbeam::{
    channel::{unbounded, Receiver, RecvError, SendError, Sender},
    select,
};

use notify;
use notify::{event::ModifyKind, recommended_watcher, RecursiveMode, Watcher};
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use crate::app::AppMessage;

enum FileWatcherMessage {
    FilePath(Option<PathBuf>),
}

#[derive(Debug)]
pub enum FileWatcherError {
    Watcher(notify::Error),
    File(io::Error),
}

impl Display for FileWatcherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FileWatcherError::Watcher(e) => write!(f, "Watcher error: {}", e),
            FileWatcherError::File(e) => write!(f, "File error: {}", e),
        }
    }
}

struct FileWatcher {
    app: Sender<AppMessage>,
    receiver: Receiver<FileWatcherMessage>,
    interval: Duration,
    file_path: Option<PathBuf>,
}

impl FileWatcher {
    pub fn new(
        app: Sender<AppMessage>,
        receiver: Receiver<FileWatcherMessage>,
        interval: Duration,
    ) -> Self {
        Self {
            app,
            receiver,
            interval,
            file_path: None,
        }
    }

    pub fn run(&mut self) -> Result<(), RecvError> {
        let (watch_sender, watch_receiver) = unbounded();

        let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
            let event = res.unwrap();
            match event.kind {
                notify::EventKind::Modify(ModifyKind::Data(_)) => {
                    watch_sender.send(event.paths).unwrap();
                }
                _ => {}
            }
        })
        .unwrap();

        let (mut _content_sender, mut _content_receiver) = unbounded::<io::Result<String>>();
        let (mut _watch_sender, mut _watch_receiver) = unbounded::<()>();

        loop {
            select! {
                recv(self.receiver) -> msg => {
                    match msg? {
                        FileWatcherMessage::FilePath(file_path) => {
                        (_content_sender, _content_receiver) = unbounded();
                        (_watch_sender, _watch_receiver) = unbounded::<()>();


                        if let Some(p) = &self.file_path {
                            watcher.unwatch(p).expect(format!("Failed to unwatch {:?}", p).as_str());
                            self.file_path = None;
                        }
                        if let Some(p) = file_path {
                            let res = watcher.watch(Path::new(&p),RecursiveMode::NonRecursive);
                            match res {
                                Ok(_) => {
                                    self.file_path = Some(p.clone());
                                    let i = self.interval.clone();
                                    thread::spawn(move || FileReader::new(_content_sender, _watch_receiver, p, i).run());
                                }
                                Err(e) => {
                                    self.app.send(AppMessage::OutputFile(Err(FileWatcherError::Watcher(e)))).unwrap();
                                }
                        }
                        } else {
                            _content_sender.send(Ok("".to_string())).unwrap();
                        }
                        }
                    }
                }
                recv(watch_receiver) -> _ => {_watch_sender.send(()).unwrap();}
                recv(_content_receiver) -> msg => {
                    self.app.send(AppMessage::OutputFile(msg.unwrap().map_err(|e| FileWatcherError::File(e)))).unwrap();
            }
            }
        }
    }
}

#[derive(Debug)]
pub struct FileWatcherHandle {
    sender: Sender<FileWatcherMessage>,
    file_path: Option<PathBuf>,
}

impl FileWatcherHandle {
    pub fn new(app: Sender<AppMessage>, interval: Duration) -> Self {
        let (sender, receiver) = unbounded();
        let mut actor = FileWatcher::new(app, receiver, interval);
        thread::spawn(move || actor.run());
        Self {
            sender,
            file_path: None,
        }
    }
    pub fn set_file_path(&mut self, file_path: Option<PathBuf>) {
        if self.file_path != file_path {
            self.file_path = file_path.clone();
            self.sender
                .send(FileWatcherMessage::FilePath(file_path))
                .unwrap();
        }
    }
}

struct FileReader {
    content_sender: Sender<io::Result<String>>,
    receiver: Receiver<()>,
    file_path: PathBuf,
    interval: Duration,
    pos: u64,
    content: String,
}

impl FileReader {
    pub fn new(
        content_sender: Sender<io::Result<String>>,
        receiver: Receiver<()>,
        file_path: PathBuf,
        interval: Duration,
    ) -> Self {
        Self {
            content_sender,
            receiver,
            file_path,
            interval,
            pos: 0,
            content: "".to_string(),
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        loop {
            // run update in a loop, either send back the file contents, or an error
            self.update().map_err(|_| ())?;
            select! {
                recv(self.receiver) -> msg => {
                    msg.map_err(|_| ())?;
                }
                default(self.interval) => {}
            }
        }
    }

    fn update(&mut self) -> Result<(), SendError<io::Result<String>>> {
        let s = File::open(&self.file_path).and_then(|mut f| {
            // update the position in the file, so we just
            self.pos = f.seek(io::SeekFrom::Start(self.pos))?;
            // advance the position by the number of bytes read from the file
            self.pos += f.read_to_string(&mut self.content)? as u64;
            Ok(self.content.clone())
        });
        // send the entire self.content string with the new stuff back on the channel

        self.content_sender.send(s)
    }
}
