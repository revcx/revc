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