# ![revc](doc/REVC.png) [![Actions Status](https://github.com/revcx/revc/workflows/revc/badge.svg?branch=master)](https://github.com/revcx/revc/actions) 

The fastest and safest EVC encoder and decoder

# Roadmap

- [ ] 0.1 Translation:
  - [x] Translate ETM baseline decoder from C to Rust
  - [ ] Translate ETM baseline encoder from C to Rust		 
- [ ] 0.2 Optimization:
  - [ ] profiling and benchmarking
  - [ ] rust safe code optimization
  - [ ] assembly optimization
    - [ ] armeabi-v7a
    - [ ] arm64-v8a
    - [ ] x86
    - [ ] x86_64  
  - [ ] multi-threading optimization
- [ ] 0.3 Modernization
  - [ ] rate control
  - [ ] practical usecases: RTC, Live Streaming, VOD, etc


# Usage

* run encoder without trace
  * cargo run run --bin revce -- -i tools/foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q 27 -r tools/tmp/rec.yuv --keyint 8 --ref_pic_gap_length 8 --skip 0 --disable_dbf --inter_slice_type 1 -o tools/tmp/test.evc -v
* run encoder with trace
  * cargo run --bin revce --features "trace,trace_coef,trace_resi,trace_reco,trace_cudata" -- -i tools/foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q 27 -r tools/tmp/rec.yuv --keyint 8 --ref_pic_gap_length 8 --skip 0 --disable_dbf --inter_slice_type 1 -o tools/tmp/test.evc -v
* run decoder without trace
  * cargo run --bin revcd -- -i test_ld_p.evc -o test.yuv -v
* run decoder with trace
  * cargo run --features "trace,trace_resi,trace_pred,trace_reco,trace_dbf" --bin revcd -- -i test_ld_p.evc -o test.yuv -v


