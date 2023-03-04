rm -rf ./img
mkdir ./img
w="a b c d e f g h j k l m n o p q r s t u x w y z"
for i in `seq 1 9` $w; do
    cargo run --bin pop_res --release -- bl320 --landtype=$i > img/$i.bmp
done
