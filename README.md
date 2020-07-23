# ![revc](doc/REVC.png) [![Actions Status](https://github.com/revcx/revc/workflows/revc/badge.svg?branch=master)](https://github.com/revcx/revc/actions) 

Rust Essential Video Coding (MPEG-5 EVC baseline)

# Roadmap

- [ ] 0.1 Translation: ETM baseline from C to Rust
  - [x] Translate ETM baseline decoder from C to Rust
  - [ ] Translate ETM baseline encoder from C to Rust		 
- [ ] 0.2 Modernization: re-architect REVC to revce/revcd, like rav1e/dav1d
- [ ] 0.3 Optimization: multi-threading and assembly
  - [ ] multi-threading
  - [ ] assembly
    - [ ] armeabi-v7a
      - armeabi
      - Thumb-2
      - VFPv3-D16
    - [ ] arm64-v8a
      - AArch64
    - [ ] assembly for x86
      - x86 (IA-32)
      - MMX
      - SSE/2/3
      - SSSE3
    - [ ] assembly for x86_64
      - x86-64
      - MMX
      - SSE/2/3
      - SSSE3
      - SSE4.1, 4.2
      - POPCNT

# Usage

* run decoder without trace
  * cargo run --bin revcd -- -i test_ld_p.evc -o test.yuv -v
* run decoder with trace
  * cargo run --features "trace,trace_resi,trace_pred,trace_reco,trace_dbf" --bin revcd -- -i test_ld_p.evc -o test.yuv -v


