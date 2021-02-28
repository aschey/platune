use act_zero::*;
struct SongQueue {
    songs: Vec<String>,
}
impl Actor for SongQueue {}

impl SongQueue {
    pub fn set_queue(&mut self, queue: Vec<String>) {
        self.songs = queue;
    }
}
