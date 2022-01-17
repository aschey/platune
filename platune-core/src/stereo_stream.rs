pub(crate) struct StereoStream {
    inner: Box<dyn Iterator<Item = [f64; 2]>>,
    current: Option<[f64; 2]>,
    position: usize,
}

impl StereoStream {
    pub(crate) fn new(mut inner: Box<dyn Iterator<Item = [f64; 2]>>) -> Self {
        let current = inner.next();
        Self {
            inner,
            current,
            position: 0,
        }
    }
}

impl Iterator for StereoStream {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(current) => {
                let next = current[self.position];
                self.position += 1;
                if self.position == 2 {
                    self.position = 0;
                    self.current = self.inner.next();
                }
                Some(next)
            }
            None => None,
        }
    }
}
