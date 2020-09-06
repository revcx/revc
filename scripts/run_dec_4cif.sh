#!/bin/sh

# ./ffmpeg -i foreman_qcif.y4m -vframes 8 -f yuv4mpegpipe crew_4cif_30fps.y4m
# ./ffmpeg -i foreman_qcif.y4m -vframes 8 -vf scale=16x16 foreman_mb8.yuv

for QP in 22 #27 32 37
do
./evca_decoder -i ./data/test_4cif_ld_i_q${QP}_etm.evc                      -o ./tmp/test_4cif_ld_i_q${QP}_etm_dec.yuv
./evca_decoder -i ./data/test_4cif_ld_p_q${QP}_etm.evc                      -o ./tmp/test_4cif_ld_p_q${QP}_etm_dec.yuv
./evca_decoder -i ./data/test_4cif_ld_b_q${QP}_etm.evc                      -o ./tmp/test_4cif_ld_b_q${QP}_etm_dec.yuv

for BFRM in 1 3 7 15
do
./evca_decoder -i ./data/test_4cif_ra_b${BFRM}_q${QP}_etm.evc          -o ./tmp/test_4cif_ra_b${BFRM}_q${QP}_etm_dec.yuv
done
done

for QP in 22 #27 32 37
do
cargo run --bin revcd --release -- -i ./data/test_4cif_ld_i_q${QP}_etm.evc                     -o ./tmp/test_4cif_ld_i_q${QP}_revc_dec.yuv -v
cargo run --bin revcd --release -- -i ./data/test_4cif_ld_p_q${QP}_etm.evc                     -o ./tmp/test_4cif_ld_p_q${QP}_revc_dec.yuv -v
cargo run --bin revcd --release -- -i ./data/test_4cif_ld_b_q${QP}_etm.evc                     -o ./tmp/test_4cif_ld_b_q${QP}_revc_dec.yuv -v

for BFRM in 1 3 7 15
do
cargo run --bin revcd --release -- -i ./data/test_4cif_ra_b${BFRM}_q${QP}_etm.evc         -o ./tmp/test_4cif_ra_b${BFRM}_q${QP}_revc_dec.yuv -v
done
done

for QP in 22 #27 32 37
do
md5sum -b                     ./tmp/test_4cif_ld_i_q${QP}_etm_dec.yuv          | awk '{print $1,"./tmp/test_4cif_ld_i_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_4cif_ld_i_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_4cif_ld_p_q${QP}_etm_dec.yuv          | awk '{print $1,"./tmp/test_4cif_ld_p_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_4cif_ld_p_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_4cif_ld_b_q${QP}_etm_dec.yuv          | awk '{print $1,"./tmp/test_4cif_ld_b_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_4cif_ld_b_q${QP}_revc_yuv.txt

for BFRM in 1 3 7 15
do
md5sum -b                     ./tmp/test_4cif_ra_b${BFRM}_q${QP}_etm_dec.yuv   | awk '{print $1,"./tmp/test_4cif_ra_b'${BFRM}'_q'${QP}'_revc_dec.yuv"}'  > ./tmp/test_4cif_ra_b${BFRM}_q${QP}_revc_yuv.txt
done
done

for QP in 22 #27 32 37
do
md5sum -c                    ./tmp/test_4cif_ld_i_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_4cif_ld_p_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_4cif_ld_b_q${QP}_revc_yuv.txt

for BFRM in 1 3 7 15
do
md5sum -c                    ./tmp/test_4cif_ra_b${BFRM}_q${QP}_revc_yuv.txt
done
done