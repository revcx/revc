#!/bin/sh

for QP in 22 27 32 37
do
./evca_decoder.exe        -i ./data/test_ld_p_nodb_q${QP}.evc                     -o ./tmp/test_ld_p_nodb_q${QP}_evca.yuv -v 0
md5sum -b                     ./tmp/test_ld_p_nodb_q${QP}_evca.yuv | awk '{print $1,"./tmp/test_ld_p_nodb_q'${QP}'_revc.yuv"}' > ./data/test_ld_p_nodb_q${QP}.txt

./evca_decoder.exe        -i ./data/test_ld_b_nodb_q${QP}.evc                     -o ./tmp/test_ld_b_nodb_q${QP}_evca.yuv -v 0
md5sum -b                     ./tmp/test_ld_b_nodb_q${QP}_evca.yuv | awk '{print $1,"./tmp/test_ld_b_nodb_q'${QP}'_revc.yuv"}' > ./data/test_ld_b_nodb_q${QP}.txt

./evca_decoder.exe        -i ./data/test_ra_b_nodb_q${QP}.evc                     -o ./tmp/test_ra_b_nodb_q${QP}_evca.yuv -v 0
md5sum -b                     ./tmp/test_ra_b_nodb_q${QP}_evca.yuv | awk '{print $1,"./tmp/test_ra_b_nodb_q'${QP}'_revc.yuv"}' > ./data/test_ra_b_nodb_q${QP}.txt

./evca_decoder.exe        -i ./data/test_ld_p_q${QP}.evc                          -o ./tmp/test_ld_p_q${QP}_evca.yuv      -v 0
md5sum -b                     ./tmp/test_ld_p_q${QP}_evca.yuv      | awk '{print $1,"./tmp/test_ld_p_q'${QP}'_revc.yuv"}'      > ./data/test_ld_p_q${QP}.txt

./evca_decoder.exe        -i ./data/test_ld_b_q${QP}.evc                          -o ./tmp/test_ld_b_q${QP}_evca.yuv      -v 0
md5sum -b                     ./tmp/test_ld_b_q${QP}_evca.yuv      | awk '{print $1,"./tmp/test_ld_b_q'${QP}'_revc.yuv"}'      > ./data/test_ld_b_q${QP}.txt

./evca_decoder.exe        -i ./data/test_ra_b_q${QP}.evc                          -o ./tmp/test_ra_b_q${QP}_evca.yuv      -v 0
md5sum -b                     ./tmp/test_ra_b_q${QP}_evca.yuv      | awk '{print $1,"./tmp/test_ra_b_q'${QP}'_revc.yuv"}'      > ./data/test_ra_b_q${QP}.txt
done


# md5sum -b  ./tmp/foreman_mb8_nodb.evc > ./data/foreman_mb8_nodb.txt
# md5sum -b  ./tmp/foreman_qcif8_nodb.evc > ./data/foreman_qcif8_nodb.txt
