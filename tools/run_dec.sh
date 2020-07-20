#!/bin/sh

for QP in 22 27 32 37
do
../target/debug/revcd.exe -i ./data/test_ld_p_nodb_q${QP}.evc   -o ./tmp/test_ld_p_nodb_q${QP}_revc.yuv
md5sum -c                    ./data/test_ld_p_nodb_q${QP}.txt

../target/debug/revcd.exe -i ./data/test_ld_b_nodb_q${QP}.evc   -o ./tmp/test_ld_b_nodb_q${QP}_revc.yuv
md5sum -c                    ./data/test_ld_b_nodb_q${QP}.txt

../target/debug/revcd.exe -i ./data/test_ra_b_nodb_q${QP}.evc   -o ./tmp/test_ra_b_nodb_q${QP}_revc.yuv
md5sum -c                    ./data/test_ra_b_nodb_q${QP}.txt

../target/debug/revcd.exe -i ./data/test_ld_p_q${QP}.evc        -o ./tmp/test_ld_p_q${QP}_revc.yuv
md5sum -c                    ./data/test_ld_p_q${QP}.txt

../target/debug/revcd.exe -i ./data/test_ld_b_q${QP}.evc        -o ./tmp/test_ld_b_q${QP}_revc.yuv
md5sum -c                    ./data/test_ld_b_q${QP}.txt

../target/debug/revcd.exe -i ./data/test_ra_b_q${QP}.evc        -o ./tmp/test_ra_b_q${QP}_revc.yuv
md5sum -c                    ./data/test_ra_b_q${QP}.txt
done




