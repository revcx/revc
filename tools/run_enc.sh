#!/bin/sh

./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -o test_ld_p.evc --config ./cfg/encoder_lowdelay_P_baseline.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -o test_ld_b.evc --config ./cfg/encoder_lowdelay_baseline.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -o test_ra_b.evc --config ./cfg/encoder_randomaccess_baseline.cfg

./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -o test_ld_p_nodb.evc --config ./cfg/encoder_lowdelay_P_baseline_nodb.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -o test_ld_b_nodb.evc --config ./cfg/encoder_lowdelay_baseline_nodb.cfg
./evca_encoder.exe -i foreman_qcif8.yuv -w 176 -h 144 -z 30 -f 8 -o test_ra_b_nodb.evc --config ./cfg/encoder_randomaccess_baseline_nodb.cfg

rm ./enc_trace.txt