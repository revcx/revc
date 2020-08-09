# ![revc](doc/REVC.png) [![Actions Status](https://github.com/revcx/revc/workflows/revc/badge.svg?branch=master)](https://github.com/revcx/revc/actions) 

The fastest and safest EVC encoder and decoder

# Roadmap

- [ ] 0.1 Translation: ETM baseline from C to Rust
  - [x] Translate ETM baseline decoder from C to Rust
  - [ ] Translate ETM baseline encoder from C to Rust		 
- [ ] 0.2 Modernization: re-architect REVC to revce/revcd, like rav1e/dav1d
- [ ] 0.3 Optimization: multi-threading and assembly
  - [ ] multi-threading
  - [ ] assembly
    - [ ] armeabi-v7a
    - [ ] arm64-v8a
    - [ ] x86
    - [ ] x86_64

# Usage

* run encoder without trace
  * cargo run --bin revce -- -i tools/foreman_mb8.yuv -w 16 -h 16 -z 30 -f 1 -q 27 -r tools/tmp/rec.yuv --ref_pic_gap_length 8 -o tools/tmp/test.evc -v
* run encoder with trace
  * cargo run --bin revce --features "trace" -- -i tools/foreman_mb8.yuv -w 16 -h 16 -z 30 -f 1 -q 27 -r tools/tmp/rec.yuv --ref_pic_gap_length 8 -o tools/tmp/test.evc -v
* run decoder without trace
  * cargo run --bin revcd -- -i test_ld_p.evc -o test.yuv -v
* run decoder with trace
  * cargo run --features "trace,trace_resi,trace_pred,trace_reco,trace_dbf" --bin revcd -- -i test_ld_p.evc -o test.yuv -v


