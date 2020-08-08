#!/bin/sh

for QP in 22 27 32 37
do
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -o ./data/test_ld_p_q${QP}.evc --config ./cfg/encoder_lowdelay_P_baseline.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -o ./data/test_ld_b_q${QP}.evc --config ./cfg/encoder_lowdelay_baseline.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -o ./data/test_ra_b_q${QP}.evc --config ./cfg/encoder_randomaccess_baseline.cfg

./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -o ./data/test_ld_p_nodb_q${QP}.evc --config ./cfg/encoder_lowdelay_P_baseline_nodb.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -o ./data/test_ld_b_nodb_q${QP}.evc --config ./cfg/encoder_lowdelay_baseline_nodb.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -q ${QP} -o ./data/test_ra_b_nodb_q${QP}.evc --config ./cfg/encoder_randomaccess_baseline_nodb.cfg
done


# ./ffmpeg.exe -i foreman_qcif.y4m -vframes 8 -f yuv4mpegpipe foreman_qcif8.y4m
# ./ffmpeg.exe -i foreman_qcif.y4m -vframes 8 -vf scale=16x16 foreman_mb8.yuv

# ./evca_encoder.exe                          -i       foreman_mb8.yuv -w 16 -h 16 -z 30 -f 1 -q 27 -r rec.yuv -o test_ld_p_q27.evc --config ../../cfg/encoder_lowdelay_P_baseline.cfg
# cargo run --bin revce --features "trace" -- -i tools/foreman_mb8.yuv -w 16 -h 16 -z 30 -f 1 -q 27 -r rec.yuv --ref_pic_gap_length 8 -o tools/tmp/test.evc -v