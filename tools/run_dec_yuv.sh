#!/bin/sh

for QP in 22 27 32 37
do

./evca_decoder.exe        -i test_ld_p_nodb_q${QP}.evc                     -o tmp/test_ld_p_nodb_q${QP}_evca.yuv -v 1
../target/debug/revcd.exe -i test_ld_p_nodb_q${QP}.evc                     -o tmp/test_ld_p_nodb_q${QP}_revc.yuv -v
md5sum -b                tmp/test_ld_p_nodb_q${QP}_evca.yuv | awk '{print $1,"tmp/test_ld_p_nodb_q'${QP}'_revc.yuv"}' > tmp/test_ld_p_nodb_q${QP}.txt
md5sum -c tmp/test_ld_p_nodb_q${QP}.txt

./evca_decoder.exe        -i test_ld_p_q${QP}.evc                     -o tmp/test_ld_p_q${QP}_evca.yuv -v 1
../target/debug/revcd.exe -i test_ld_p_q${QP}.evc                     -o tmp/test_ld_p_q${QP}_revc.yuv -v
md5sum -b                tmp/test_ld_p_q${QP}_evca.yuv | awk '{print $1,"tmp/test_ld_p_q'${QP}'_revc.yuv"}' > tmp/test_ld_p_q${QP}.txt
md5sum -c                tmp/test_ld_p_q${QP}.txt

./evca_decoder.exe        -i test_ld_b_nodb_q${QP}.evc                     -o tmp/test_ld_b_nodb_q${QP}_evca.yuv -v 1
../target/debug/revcd.exe -i test_ld_b_nodb_q${QP}.evc                     -o tmp/test_ld_b_nodb_q${QP}_revc.yuv -v
md5sum -b                tmp/test_ld_b_nodb_q${QP}_evca.yuv | awk '{print $1,"tmp/test_ld_b_nodb_q'${QP}'_revc.yuv"}' > tmp/test_ld_b_nodb_q${QP}.txt
md5sum -c                tmp/test_ld_b_nodb_q${QP}.txt

./evca_decoder.exe        -i test_ld_b_q${QP}.evc                     -o tmp/test_ld_b_q${QP}_evca.yuv -v 1
../target/debug/revcd.exe -i test_ld_b_q${QP}.evc                     -o tmp/test_ld_b_q${QP}_revc.yuv -v
md5sum -b                tmp/test_ld_b_q${QP}_evca.yuv | awk '{print $1,"tmp/test_ld_b_q'${QP}'_revc.yuv"}' > tmp/test_ld_b_q${QP}.txt
md5sum -c                tmp/test_ld_b_q${QP}.txt

./evca_decoder.exe        -i test_ra_b_nodb_q${QP}.evc                     -o tmp/test_ra_b_nodb_q${QP}_evca.yuv -v 1
../target/debug/revcd.exe -i test_ra_b_nodb_q${QP}.evc                     -o tmp/test_ra_b_nodb_q${QP}_revc.yuv -v
md5sum -b                tmp/test_ra_b_nodb_q${QP}_evca.yuv | awk '{print $1,"tmp/test_ra_b_nodb_q'${QP}'_revc.yuv"}' > tmp/test_ra_b_nodb_q${QP}.txt
md5sum -c                tmp/test_ra_b_nodb_q${QP}.txt

./evca_decoder.exe        -i test_ra_b_q${QP}.evc                     -o tmp/test_ra_b_q${QP}_evca.yuv -v 1
../target/debug/revcd.exe -i test_ra_b_q${QP}.evc                     -o tmp/test_ra_b_q${QP}_revc.yuv -v
md5sum -b                tmp/test_ra_b_q${QP}_evca.yuv | awk '{print $1,"tmp/test_ra_b_q'${QP}'_revc.yuv"}' > tmp/test_ra_b_q${QP}.txt
md5sum -c                tmp/test_ra_b_q${QP}.txt

done




