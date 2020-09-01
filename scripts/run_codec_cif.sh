#!/bin/sh

# ./ffmpeg -i foreman_qcif.y4m -vframes 8 -f yuv4mpegpipe foreman_cif8.y4m
# ./ffmpeg -i foreman_qcif.y4m -vframes 8 -vf scale=16x16 foreman_mb8.yuv

for QP in 22 27 32 37
do
./evca_encoder -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ld_i_q${QP}_etm.yuv -o ./tmp/test_cif_ld_i_q${QP}_etm.evc --config ./cfg/encoder_lowdelay_P_baseline.cfg -p 1
./evca_encoder -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ld_p_q${QP}_etm.yuv -o ./tmp/test_cif_ld_p_q${QP}_etm.evc --config ./cfg/encoder_lowdelay_P_baseline.cfg
./evca_encoder -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ld_b_q${QP}_etm.yuv -o ./tmp/test_cif_ld_b_q${QP}_etm.evc --config ./cfg/encoder_lowdelay_baseline.cfg

./evca_decoder -i ./tmp/test_cif_ld_i_q${QP}_etm.evc                      -o ./tmp/test_cif_ld_i_q${QP}_etm_dec.yuv
./evca_decoder -i ./tmp/test_cif_ld_p_q${QP}_etm.evc                      -o ./tmp/test_cif_ld_p_q${QP}_etm_dec.yuv
./evca_decoder -i ./tmp/test_cif_ld_b_q${QP}_etm.evc                      -o ./tmp/test_cif_ld_b_q${QP}_etm_dec.yuv

for BFRM in 1 3 7 15
do
./evca_encoder -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm.yuv -o ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm.evc --config ./cfg/encoder_randomaccess_baseline_bn.cfg --max_b_frames ${BFRM}
./evca_decoder -i ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm.evc          -o ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm_dec.yuv
done
done

for QP in 22 27 32 37
do
cargo run --bin revce --release -- -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ld_i_q${QP}_revc.yuv -o ./tmp/test_cif_ld_i_q${QP}_revc.evc --ref_pic_gap_length 8 --inter_slice_type 1 -v -p 1
cargo run --bin revce --release -- -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ld_p_q${QP}_revc.yuv -o ./tmp/test_cif_ld_p_q${QP}_revc.evc --ref_pic_gap_length 8 --inter_slice_type 1 -v
cargo run --bin revce --release -- -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ld_b_q${QP}_revc.yuv -o ./tmp/test_cif_ld_b_q${QP}_revc.evc --ref_pic_gap_length 8 --inter_slice_type 0 -v

cargo run --bin revcd --release -- -i ./tmp/test_cif_ld_i_q${QP}_revc.evc                     -o ./tmp/test_cif_ld_i_q${QP}_revc_dec.yuv -v
cargo run --bin revcd --release -- -i ./tmp/test_cif_ld_p_q${QP}_revc.evc                     -o ./tmp/test_cif_ld_p_q${QP}_revc_dec.yuv -v
cargo run --bin revcd --release -- -i ./tmp/test_cif_ld_b_q${QP}_revc.evc                     -o ./tmp/test_cif_ld_b_q${QP}_revc_dec.yuv -v

for BFRM in 1 3 7 15
do
cargo run --bin revce --release -- -i foreman_cif8.yuv -w 352 -h 288 -z 30 -f 8 -q ${QP} -r ./tmp/test_cif_ra_b${BFRM}_q${QP}_revc.yuv -o ./tmp/test_cif_ra_b${BFRM}_q${QP}_revc.evc --max_b_frames ${BFRM} --inter_slice_type 0 -v
cargo run --bin revcd --release -- -i ./tmp/test_cif_ra_b${BFRM}_q${QP}_revc.evc         -o ./tmp/test_cif_ra_b${BFRM}_q${QP}_revc_dec.yuv -v
done
done

for QP in 22 27 32 37
do
md5sum -b                     ./tmp/test_cif_ld_i_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_i_q'${QP}'_revc.yuv"}'               > ./tmp/test_cif_ld_i_q${QP}_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_i_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_i_q'${QP}'_etm_dec.yuv"}'            > ./tmp/test_cif_ld_i_q${QP}_etm_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_i_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_i_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_cif_ld_i_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_i_q${QP}_etm.evc          | awk '{print $1,"./tmp/test_cif_ld_i_q'${QP}'_revc.evc"}'               > ./tmp/test_cif_ld_i_q${QP}_evc.txt

md5sum -b                     ./tmp/test_cif_ld_p_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_p_q'${QP}'_revc.yuv"}'               > ./tmp/test_cif_ld_p_q${QP}_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_p_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_p_q'${QP}'_etm_dec.yuv"}'            > ./tmp/test_cif_ld_p_q${QP}_etm_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_p_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_p_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_cif_ld_p_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_p_q${QP}_etm.evc          | awk '{print $1,"./tmp/test_cif_ld_p_q'${QP}'_revc.evc"}'               > ./tmp/test_cif_ld_p_q${QP}_evc.txt

md5sum -b                     ./tmp/test_cif_ld_b_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_b_q'${QP}'_revc.yuv"}'               > ./tmp/test_cif_ld_b_q${QP}_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_b_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_b_q'${QP}'_etm_dec.yuv"}'            > ./tmp/test_cif_ld_b_q${QP}_etm_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_b_q${QP}_etm.yuv          | awk '{print $1,"./tmp/test_cif_ld_b_q'${QP}'_revc_dec.yuv"}'           > ./tmp/test_cif_ld_b_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_cif_ld_b_q${QP}_etm.evc          | awk '{print $1,"./tmp/test_cif_ld_b_q'${QP}'_revc.evc"}'               > ./tmp/test_cif_ld_b_q${QP}_evc.txt

for BFRM in 1 3 7 15
do
md5sum -b                     ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm.yuv   | awk '{print $1,"./tmp/test_cif_ra_b'${BFRM}'_q'${QP}'_revc.yuv"}'      > ./tmp/test_cif_ra_b${BFRM}_q${QP}_yuv.txt
md5sum -b                     ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm.yuv   | awk '{print $1,"./tmp/test_cif_ra_b'${BFRM}'_q'${QP}'_etm_dec.yuv"}'   > ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm_yuv.txt
md5sum -b                     ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm.yuv   | awk '{print $1,"./tmp/test_cif_ra_b'${BFRM}'_q'${QP}'_revc_dec.yuv"}'  > ./tmp/test_cif_ra_b${BFRM}_q${QP}_revc_yuv.txt
md5sum -b                     ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm.evc   | awk '{print $1,"./tmp/test_cif_ra_b'${BFRM}'_q'${QP}'_revc.evc"}'      > ./tmp/test_cif_ra_b${BFRM}_q${QP}_evc.txt
done
done

for QP in 22 27 32 37
do
md5sum -c                    ./tmp/test_cif_ld_i_q${QP}_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_i_q${QP}_etm_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_i_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_i_q${QP}_evc.txt

md5sum -c                    ./tmp/test_cif_ld_p_q${QP}_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_p_q${QP}_etm_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_p_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_p_q${QP}_evc.txt

md5sum -c                    ./tmp/test_cif_ld_b_q${QP}_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_b_q${QP}_etm_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_b_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_cif_ld_b_q${QP}_evc.txt

for BFRM in 1 3 7 15
do
md5sum -c                    ./tmp/test_cif_ra_b${BFRM}_q${QP}_yuv.txt
md5sum -c                    ./tmp/test_cif_ra_b${BFRM}_q${QP}_etm_yuv.txt
md5sum -c                    ./tmp/test_cif_ra_b${BFRM}_q${QP}_revc_yuv.txt
md5sum -c                    ./tmp/test_cif_ra_b${BFRM}_q${QP}_evc.txt
done
done