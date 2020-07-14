#!/bin/sh

./evca_decoder.exe -i test_ld_p_nodb.evc -o tmp/test_ld_p_nodb_evca.yuv -v 1
../target/debug/revcd.exe -i test_ld_p_nodb.evc -o tmp/test_ld_p_nodb_revc.yuv -v
diff ./tmp/test_ld_p_nodb_evca.yuv ./tmp/test_ld_p_nodb_revc.yuv

./evca_decoder.exe -i test_ld_b_nodb.evc -o tmp/test_ld_b_nodb_evca.yuv -v 1
../target/debug/revcd.exe -i test_ld_b_nodb.evc -o tmp/test_ld_b_nodb_revc.yuv -v
diff ./tmp/test_ld_b_nodb_evca.yuv ./tmp/test_ld_b_nodb_revc.yuv

./evca_decoder.exe -i test_ra_b_nodb.evc -o tmp/test_ra_b_nodb_evca.yuv -v 1
../target/debug/revcd.exe -i test_ra_b_nodb.evc -o tmp/test_ra_b_nodb_revc.yuv -v
diff ./tmp/test_ra_b_nodb_evca.yuv ./tmp/test_ra_b_nodb_revc.yuv
