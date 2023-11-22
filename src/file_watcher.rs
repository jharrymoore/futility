use std::path::PathBuf;

pub fn file_watcher(file: PathBuf) -> Option<Vec<String>> {
    let contents = std::fs::read_to_string(file);
    match contents {
        Ok(contents) => {
            let lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
            Some(lines)
        }
        Err(_) => None,
    }
}
// TODO: use notify to watch for file changes, handle events in a loop insead of parsing the whole
// thing every time we switch files.
// pub fn watch_file() {
//     // Create a channel to receive events.
//     let (tx, rx) = channel();
//
//     // Create a watcher object.
//     let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
//
//     // Watch the specified file for changes.
//     watcher
//         .watch("path/to/your/file", RecursiveMode::NonRecursive)
//         .unwrap();
//
//     println!("Watching for changes...");
//
//     // Listen for events in a loop.
//     loop {
//         match rx.recv() {
//             Ok(event) => {
//                 println!("Event: {:?}", event);
//
//                 // Handle the event here (e.g., check if the event is a modification).
//                 // You can add your logic to respond to file changes.
//             }
//             Err(e) => println!("Watcher error: {:?}", e),
//         }
//     }
// }
