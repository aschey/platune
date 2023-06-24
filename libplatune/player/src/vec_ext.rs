pub(crate) trait VecExt {
    fn fill_from_deinterleaved(&mut self, channels: &[Vec<f32>]);
}

impl VecExt for Vec<f32> {
    fn fill_from_deinterleaved(&mut self, deinterleaved: &[Vec<f32>]) {
        let out_len = deinterleaved[0].len();
        self.clear();

        for i in 0..out_len {
            for chan in deinterleaved {
                self.push(chan[i]);
            }
        }
    }
}
