pub(crate) struct ChannelBuffer {
    inner: Vec<Vec<f64>>,
    capacity: usize,
    channels: usize,
}

impl ChannelBuffer {
    pub(crate) fn new(capacity: usize, channels: usize) -> Self {
        Self {
            inner: vec![Vec::with_capacity(capacity); channels],
            capacity,
            channels,
        }
    }

    fn len(&self) -> usize {
        self.inner[self.channels - 1].len()
    }

    pub(crate) fn position(&self) -> usize {
        self.inner.last().unwrap().len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.inner.last().unwrap().len() == self.inner.last().unwrap().capacity()
    }

    pub(crate) fn reset(&mut self) {
        for chan in &mut self.inner {
            chan.clear();
        }
    }

    pub(crate) fn inner(&self) -> &[Vec<f64>] {
        &self.inner
    }

    pub(crate) fn silence_remainder(&mut self) {
        while self.len() < self.capacity {
            for chan in &mut self.inner {
                chan.push(0.0);
            }
        }
    }

    pub(crate) fn fill_from_slice(&mut self, data: &[f64]) -> usize {
        let mut i = 0;
        while self.len() < self.capacity && i < data.len() {
            for chan in &mut self.inner {
                chan.push(data[i]);
                i += 1;
            }
        }
        i
    }
}