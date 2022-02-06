use ignore::WalkBuilder;
use ignore::WalkState::*;
use katatsuki::Track;
use lewton::inside_ogg::OggStreamReader;
use rayon::prelude::*;
use std::fs::File;
use std::time::Instant;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use walkdir::WalkDir;
fn main() {
    let now = Instant::now();
    // WalkDir::new("/home/aschey/windows/shared_files/Music")
    //     .into_iter()
    //     .par_iter()
    //     .map(|i| {});
    let walker = WalkBuilder::new("/home/aschey/windows/shared_files/Music")
        .threads(12)
        .build_parallel();
    walker.run(|| {
        Box::new(move |result| {
            if let Ok(result) = result {
                let path = result.into_path();
                if path.is_file() {
                    let name = path.extension().unwrap_or_default();
                    let ext = &name.to_str().unwrap_or_default().to_lowercase()[..];
                    match &name.to_str().unwrap_or_default().to_lowercase()[..] {
                        "mp3" => {
                            let t = id3::Tag::read_from_path(&path);
                        }
                        "ogg" => {
                            let file = File::open(path).unwrap();
                            let source = OggStreamReader::new(file).unwrap();
                            for (key, value) in source.comment_hdr.comment_list {}
                        }
                        "m4a" => {
                            let mut tag = mp4ameta::Tag::read_from_path(path).unwrap();
                        }
                        _ => {} // "mp3" | "m4a" | "ogg" | "wav" | "flac" | "aac" => {
                                //     let mut hint = Hint::new();
                                //     hint.with_extension(ext);
                                //     let mss = MediaSourceStream::new(
                                //         Box::new(File::open(path).unwrap()),
                                //         Default::default(),
                                //     );
                                //     if let Ok(mut probed) = symphonia::default::get_probe().format(
                                //         &hint,
                                //         mss,
                                //         &Default::default(),
                                //         &Default::default(),
                                //     ) {
                                //         let tags = probed.format.metadata().current().map(|c| c.tags());
                                //     }

                                //     // let tag_result = Track::from_path(&path, None);
                                //     // match tag_result {
                                //     //     Err(e) => {
                                //     //         println!("{:?}", e);
                                //     //     }
                                //     //     Ok(tag) => {
                                //     //         // song_metadata = Some(tag);
                                //     //     }
                                //     // }
                                // }

                                // _ => {}
                    };

                    //let t = Track::from_path(&path, None).unwrap();
                }
            }
            Continue
        })
    });

    // {
    //     let path = entry.into_path();
    //     if path.is_file() {
    //         let name = path.extension().unwrap_or_default();
    //         match &name.to_str().unwrap_or_default().to_lowercase()[..] {
    //             "mp3" | "m4a" | "ogg" | "wav" | "flac" | "aac" => {
    //                 let tag_result = Track::from_path(&path, None);
    //                 match tag_result {
    //                     Err(e) => {
    //                         println!("{:?}", e);
    //                     }
    //                     Ok(tag) => {
    //                         // song_metadata = Some(tag);
    //                     }
    //                 }
    //             }

    //             _ => {}
    //         }
    //         //let t = Track::from_path(&path, None).unwrap();
    //     }

    //     //println!("{}", entry.unwrap().path().display());
    // }
    println!("{:?}", now.elapsed());
}
