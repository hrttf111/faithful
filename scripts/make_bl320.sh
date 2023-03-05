rm -rf ./img
mkdir ./img
w="a b c d e f g h j k l m n o p q r s t u x w y z"
for i in `seq 1 9` $w; do
    cargo run --bin pop_res --release -- bl320 --landtype=$i > img/$i.bmp
done
#cargo run --bin pop_res --release -- bl160 --landtype=1 16 16 > img/1_16_16.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 32 32 > img/1_32_32.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 48 48 > img/1_48_48.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 64 32 > img/1_64_32.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 64 64 > img/1_64_64.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 80 80 > img/1_80_16.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 96 96 > img/1_96_16.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 128 32 > img/1_128_32.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 128 64 > img/1_128_64.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=1 128 128 > img/1_128_128.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=2 128 128 > img/2_128_128.bmp
#cargo run --bin pop_res --release -- bl160 --landtype=0 128 128 > img/0_128_128.bmp
