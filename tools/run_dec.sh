#!/bin/sh

# run --features "trace,trace_resi,trace_pred,trace_reco,trace_dbf" --bin revcd -- -i C:\msys64\home\yuliu\revc\tools\test_ld_p.evc -o C:\msys64\home\yuliu\revc\tools\tmp\test.yuv -v

./evca_decoder.exe -i test_ld_p.evc -o tmp/test_ld_p_evca.yuv -v 1
dos2unix ./dec_trace.txt
mv ./dec_trace.txt ./tmp/dec_trace_ld_p_evac.txt
../target/debug/revcd.exe -i test_ld_p.evc -o tmp/test_ld_p_revc.yuv -v
mv ./dec_trace.txt ./tmp/dec_trace_ld_p_revc.txt
diff ./tmp/dec_trace_ld_p_evac.txt ./tmp/dec_trace_ld_p_revc.txt

./evca_decoder.exe -i test_ld_b.evc -o tmp/test_ld_b_evca.yuv -v 1
dos2unix ./dec_trace.txt
mv ./dec_trace.txt ./tmp/dec_trace_ld_b_evac.txt
../target/debug/revcd.exe -i test_ld_b.evc -o tmp/test_ld_b_revc.yuv -v
mv ./dec_trace.txt ./tmp/dec_trace_ld_b_revc.txt
diff ./tmp/dec_trace_ld_b_evac.txt ./tmp/dec_trace_ld_b_revc.txt

./evca_decoder.exe -i test_ra_b.evc -o tmp/test_ra_b_evca.yuv -v 1
dos2unix ./dec_trace.txt
mv ./dec_trace.txt ./tmp/dec_trace_ra_b_evac.txt
../target/debug/revcd.exe -i test_ra_b.evc -o tmp/test_ra_b_revc.yuv -v
mv ./dec_trace.txt ./tmp/dec_trace_ra_b_revc.txt
diff ./tmp/dec_trace_ra_b_evac.txt ./tmp/dec_trace_ra_b_revc.txt
