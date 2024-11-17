// use std::thread;
// use std::time::Duration;
// use crate::command_handler::process_output;
// use crate::types::{HandleVec, ThreadVec};
//
// pub(crate) struct TaskManager {
//     thread_vec: ThreadVec,
//     handle_vec: HandleVec,
// }
//
// impl TaskManager {
//     pub(crate) fn init(thread_vec: ThreadVec, handle_vec: HandleVec) -> Self {
//         TaskManager { thread_vec, handle_vec }
//     }
//
//     pub(crate) fn run(&mut self) {
//         loop {
//             self.check_threads();
//             // Sleep to prevent tight looping
//             thread::sleep(Duration::from_millis(2000));
//         }
//     }
//
//     pub(crate) fn check_threads(&mut self) {
//         log::trace!("Checking threads for completeness");
//         let mut finished_threads = Vec::new();
//         log::trace!("about to lock in check_threads");
//         let mut threads = self.thread_vec.lock().unwrap();
//         log::trace!("locked in check_threads");
//
//         dbg!(&threads);
//
//         // Find indices of all completed threads
//         for (index, thread) in threads.iter().enumerate() {
//             if thread.is_finished() {
//                 log::trace!("Thread {} is done", index);
//                 finished_threads.push(index);
//             } else {
//                 log::trace!("Thread {} is not done", index);
//             }
//         }
//
//         // Remove all completed threads from thread vector
//         for index in finished_threads.into_iter().rev() {
//             log::trace!("Thread with index {} is completed", index);
//             if let Ok(result) = threads.remove(index).join() {
//                 process_output(result, &self.handle_vec)
//             } else {
//                 log::debug!("Join failed");
//             }
//         }
//         log::trace!("Check threads finished");
//     }
// }