`deew -i *thd`\
encode DDP

`deew -b 768 -i *flac`\
encode DDP@768

`deew -dm 2 -f dd -b 192 -i *.ec3`\
encode DD@192 with stereo downmixing

`deew -f dd -b 448 -in 4 -i S01`\
encode DD@448 using 4 instances (input is a folder)

`deew -f thd -i *w64`\
encode TrueHD

`deew -f dd -i *dts -k`\
`deew -f ddp -i *dts`\
encode multiple formats/bitrates while creating the temp file only once