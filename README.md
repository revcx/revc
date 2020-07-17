# revc
Rust Essential Video Coding (MPEG-5 EVC)

# Roadmap

- [ ] 1.0 Translation: ETM baseline from C to Rust
  - [x] Translate ETM baseline decoder from C to Rust
  - [ ] Translate ETM baseline encoder from C to Rust		 
- [ ] 2.0 Modernization: re-architect REVC to revce/revcd, like rav1e/dav1d
- [ ] 3.0 Optimization: ASM for x86_64, Neon for arm64 

# Usage

* run decoder without trace
  * cargo run --bin revcd -- -i test_ld_p.evc -o test.yuv -v
* run decoder with trace
  * cargo run --features "trace,trace_resi,trace_pred,trace_reco,trace_dbf" --bin revcd -- -i test_ld_p.evc -o test.yuv -v


