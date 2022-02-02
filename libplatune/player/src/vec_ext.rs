pub(crate) trait VecExt {
    fn fill_from_deinterleaved(&mut self, channels: Vec<Vec<f64>>);
}

impl VecExt for Vec<f64> {
    fn fill_from_deinterleaved(&mut self, deinterleaved: Vec<Vec<f64>>) {
        let out_len = deinterleaved[0].len();
        self.clear();

        for i in 0..out_len {
            for chan in &deinterleaved {
                self.push(chan[i]);
            }
        }
    }
}
