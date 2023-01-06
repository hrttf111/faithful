rm -rf ./img
mkdir ./img
for i in `seq 1 25`; do
    cargo run --bin pop_res --release -- globe $i > img/$i.bmp
done

MAPS_2="
79
80
82
83
84
94
100
109
110
111
112
120
127
128
131
133
134
"

for i in $MAPS_2; do
    cargo run --bin pop_res --release -- globe $i > img/$i.bmp
done
