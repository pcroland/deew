`deew -i *thd`\
DDP encode-olása

`deew -b 768 -i *flac`\
DDP@768 encode-olása

`deew -dm 2 -f dd -b 192 -i *.ec3`\
DD@192 encode-olása stereo downmixeléssel

`deew -f dd -b 448 -in 4 -i S01`\
DD@448 encode-olása 4 instance-et használva (az input egy mappa)

`deew -f thd -i *w64`\
TrueHD encode-olása

`deew -f dd -i *dts -k`\
`deew -f ddp -i *dts`\
több formátum/bitráta kódolása egy ideiglenes fájl készítésével