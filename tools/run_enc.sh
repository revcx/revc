#!/bin/sh

# ./ffmpeg.exe -i foreman_qcif.y4m -vframes 8 -f yuv4mpegpipe foreman_qcif8.y4m
# ./ffmpeg.exe -i foreman_qcif.y4m -vframes 8 -vf scale=16x16 foreman_mb8.yuv

for QP in 22 27 32 37
do
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ld_p_nodb_q${QP}_etm.yuv -o ./tmp/test_ld_p_nodb_q${QP}_etm.evc --config ./cfg/encoder_lowdelay_P_baseline_nodb.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ld_b_nodb_q${QP}_etm.yuv -o ./tmp/test_ld_b_nodb_q${QP}_etm.evc --config ./cfg/encoder_lowdelay_baseline_nodb.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ra_b_nodb_q${QP}_etm.yuv -o ./tmp/test_ra_b_nodb_q${QP}_etm.evc --config ./cfg/encoder_randomaccess_baseline_nodb.cfg

#./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ld_p_q${QP}_etm.yuv -o ./tmp/test_ld_p_q${QP}_etm.evc --config ./cfg/encoder_lowdelay_P_baseline.cfg
#./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ld_b_q${QP}_etm.yuv -o ./tmp/test_ld_b_q${QP}_etm.evc --config ./cfg/encoder_lowdelay_baseline.cfg
#./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ra_b_q${QP}_etm.yuv -o ./tmp/test_ra_b_q${QP}_etm.evc --config ./cfg/encoder_randomaccess_baseline.cfg
done

for QP in 22 27 32 37
do
cargo run --bin revce --release -- -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ld_p_nodb_q${QP}_revc.yuv -o ./tmp/test_ld_p_nodb_q${QP}_revc.evc --ref_pic_gap_length 8 --disable_dbf --inter_slice_type 1 -v
cargo run --bin revce --release -- -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ld_b_nodb_q${QP}_revc.yuv -o ./tmp/test_ld_b_nodb_q${QP}_revc.evc --ref_pic_gap_length 8 --disable_dbf --inter_slice_type 0 -v
cargo run --bin revce --release -- -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -r ./tmp/test_ra_b_nodb_q${QP}_revc.yuv -o ./tmp/test_ra_b_nodb_q${QP}_revc.evc --ref_pic_gap_length 8 --disable_dbf --inter_slice_type 0 --max_b_frames 15 -v
done

for QP in 22 27 32 37
do
md5sum -b                     ./tmp/test_ld_p_nodb_q${QP}_etm.yuv | awk '{print $1,"./tmp/test_ld_p_nodb_q'${QP}'_revc.yuv"}' > ./tmp/test_ld_p_nodb_q${QP}.txt
md5sum -b                     ./tmp/test_ld_b_nodb_q${QP}_etm.yuv | awk '{print $1,"./tmp/test_ld_b_nodb_q'${QP}'_revc.yuv"}' > ./tmp/test_ld_b_nodb_q${QP}.txt
md5sum -b                     ./tmp/test_ra_b_nodb_q${QP}_etm.yuv | awk '{print $1,"./tmp/test_ra_b_nodb_q'${QP}'_revc.yuv"}' > ./tmp/test_ra_b_nodb_q${QP}.txt

#md5sum -b                     ./tmp/test_ld_p_q${QP}_etm.yuv      | awk '{print $1,"./tmp/test_ld_p_q'${QP}'_revc.yuv"}'      > ./tmp/test_ld_p_q${QP}.txt
#md5sum -b                     ./tmp/test_ld_b_q${QP}_etm.yuv      | awk '{print $1,"./tmp/test_ld_b_q'${QP}'_revc.yuv"}'      > ./tmp/test_ld_b_q${QP}.txt
#md5sum -b                     ./tmp/test_ra_b_q${QP}_etm.yuv      | awk '{print $1,"./tmp/test_ra_b_q'${QP}'_revc.yuv"}'      > ./tmp/test_ra_b_q${QP}.txt
done

for QP in 22 27 32 37
do
md5sum -c                    ./tmp/test_ld_p_nodb_q${QP}.txt
md5sum -c                    ./tmp/test_ld_b_nodb_q${QP}.txt
md5sum -c                    ./tmp/test_ra_b_nodb_q${QP}.txt

#md5sum -c                    ./tmp/test_ld_p_q${QP}.txt
#md5sum -c                    ./tmp/test_ld_b_q${QP}.txt
#md5sum -c                    ./tmp/test_ra_b_q${QP}.txt
done