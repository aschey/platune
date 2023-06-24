pub(crate) struct ChannelBuffer {
    inner: Vec<Vec<f32>>,
    capacity: usize,
    channels: usize,
    current_chan: usize,
}

impl ChannelBuffer {
    pub(crate) fn new(capacity: usize, channels: usize) -> Self {
        Self {
            inner: vec![Vec::with_capacity(capacity); channels],
            capacity,
            channels,
            current_chan: 0,
        }
    }

    fn len(&self) -> usize {
        self.inner[self.channels - 1].len()
    }

    fn last(&self) -> &Vec<f32> {
        self.inner.last().expect("Vec should not be empty")
    }

    fn next_chan(&mut self) -> &mut Vec<f32> {
        let chan = &mut self.inner[self.current_chan];
        self.current_chan = (self.current_chan + 1) % self.channels;
        chan
    }

    pub(crate) fn position(&self) -> usize {
        self.last().len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.last().len() == self.last().capacity()
    }

    pub(crate) fn reset(&mut self) {
        for chan in &mut self.inner {
            chan.clear();
        }
        self.current_chan = 0;
    }

    pub(crate) fn inner(&self) -> &[Vec<f32>] {
        &self.inner
    }

    pub(crate) fn silence_remainder(&mut self) {
        while self.len() < self.capacity {
            self.next_chan().push(0.0);
        }
    }

    pub(crate) fn fill_from_slice(&mut self, data: &[f32]) -> usize {
        let mut i = 0;
        while self.len() < self.capacity && i < data.len() {
            self.next_chan().push(data[i]);
            i += 1;
        }
        i
    }
}
