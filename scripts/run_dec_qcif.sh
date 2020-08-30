#!/bin/sh


for QP in 22 27 32 37
do
./evca_decoder -i ./tmp/test_ld_i_q${QP}_etm.evc                      -o ./tmp/test_ld_i_q${QP}_etm_dec.yuv
./evca_decoder -i ./tmp/test_ld_p_q${QP}_etm.evc                      -o ./tmp/test_ld_p_q${QP}_etm_dec.yuv
./evca_decoder -i ./tmp/test_ld_b_q${QP}_etm.evc                      -o ./tmp/test_ld_b_q${QP}_etm_dec.yuv

for BFRM in 1 3 7 15
do
./evca_decoder -i ./tmp/test_ra_b${BFRM}_q${QP}_etm.evc               -o ./tmp/test_ra_b${BFRM}_q${QP}_etm_dec.yuv
done
done

for QP in 22 27 32 37
do
cargo run --bin revcd --release -- -i ./tmp/test_ld_i_q${QP}_revc.evc                     -o ./tmp/test_ld_i_q${QP}_revc_dec.yuv -v
cargo run --bin revcd --release -- -i ./tmp/test_ld_p_q${QP}_revc.evc                     -o ./tmp/test_ld_p_q${QP}_revc_dec.yuv -v
cargo run --bin revcd --release -- -i ./tmp/test_ld_b_q${QP}_revc.evc                     -o ./tmp/test_ld_b_q${QP}_revc_dec.yuv -v

for BFRM in 1 3 7 15
do
cargo run --bin revcd --release -- -i ./tmp/test_ra_b${BFRM}_q${QP}_revc.evc              -o ./tmp/test_ra_b${BFRM}_q${QP}_revc_dec.yuv -v
done
done

for QP in 22 27 32 37
do
md5sum -b                     ./tmp/test_ld_i_q${QP}_etm_dec.yuv          | awk '{print $1,"./tmp/test_ld_i_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_ld_i_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_ld_p_q${QP}_etm_dec.yuv          | awk '{print $1,"./tmp/test_ld_p_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_ld_p_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_ld_b_q${QP}_etm_dec.yuv          | awk '{print $1,"./tmp/test_ld_b_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_ld_b_q${QP}_revc_yuv.txt

for BFRM in 1 3 7 15
do
md5sum -b                     ./tmp/test_ra_b${BFRM}_q${QP}_etm_dec.yuv   | awk '{print $1,"./tmp/test_ra_b'${BFRM}'_q'${QP}'_revc_dec.yuv"}'  > ./tmp/test_ra_b${BFRM}_q${QP}_revc_yuv.txt
done
done

for QP in 22 27 32 37
do
md5sum -c                    ./tmp/test_ld_i_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_ld_p_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_ld_b_q${QP}_revc_yuv.txt

for BFRM in 1 3 7 15
do
md5sum -c                    ./tmp/test_ra_b${BFRM}_q${QP}_revc_yuv.txt
done
done